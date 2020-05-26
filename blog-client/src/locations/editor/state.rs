use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::{editor::M, Location, M as LocationM},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        PostMarker, Store as GlobalS, StoreOperations as GSOp, User,
    },
    shared::retry,
};
use db_models::models::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum S {
    Undetermined(PostMarker),
    New(posts::NewNoMeta),
    Old(posts::DataNoMeta, posts::Changed),
}

impl From<PostMarker> for S {
    fn from(s: PostMarker) -> Self {
        Self::Undetermined(s)
    }
}
impl S {
    pub fn to_url(&self) -> Url {
        const DEFAULT_SLUG: &'static str = "new";
        let opt_slug = match self {
            S::New(_) => None,
            S::Old(post, _) => {
                let marker: PostMarker = post.into();
                Some(marker.to_slug())
            }
            S::Undetermined(pm) => Some(pm.to_slug()),
        };
        let slug = opt_slug.as_ref().map_or(DEFAULT_SLUG, String::as_str);
        Url::new().set_path(vec!["blog", "edit", slug])
    }
    pub fn is_publishable(&self) -> bool {
        match self {
            Self::New(_) => true,
            Self::Old(post, _) => match post {
                // If not published, or archived but not deleted, allow publish button.
                posts::DataNoMeta {
                    published_at: None,
                    archived_at: None,
                    deleted_at: None,
                    ..
                }
                | posts::DataNoMeta {
                    archived_at: Some(_),
                    deleted_at: None,
                    ..
                } => true,
                _ => false,
            },
            Self::Undetermined(_) => false,
        }
    }
    pub fn old_ref(&self) -> Option<&posts::DataNoMeta> {
        match self {
            Self::Old(p, _) => Some(p),
            _ => None,
        }
    }
    pub fn update_title(&mut self, title: String) {
        match self {
            Self::Old(_, changed) => {
                changed.title = Some(title);
            }
            Self::New(post) => {
                post.title = title;
            }
            _ => (),
        }
    }
    pub fn update_body(&mut self, body: String) {
        match self {
            Self::Old(_, changed) => {
                changed.body = Some(body);
            }
            Self::New(post) => {
                post.body = body;
            }
            _ => (),
        }
    }
    pub fn update_slug(&mut self, slug: String) {
        let slug = match slug.trim() {
            "" => None,
            _ => Some(slug),
        };
        match self {
            Self::New(post) => {
                post.slug = slug;
            }
            _ => (),
        }
    }
}
impl Default for S {
    fn default() -> Self {
        Self::New(posts::NewNoMeta::default())
    }
}

impl S {
    async fn attempt_save_async_new(post: posts::NewNoMeta) -> GlobalM {
        const CREATE_POST_URL: &str = "/api/posts";
        const NEW_SAVE_MSG: retry::LogPair<'static> = retry::LogPair {
            pre_completion: "creating new post",
            post_completion: "parsing created post",
        };

        let req = Request::new(CREATE_POST_URL)
            .method(Method::Post)
            .json(&post);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_json_with_retry(
            req,
            &NEW_SAVE_MSG,
            None,
        ).await;
        match res {
            Err(_) => GlobalM::NoOp,
            Ok(obj) => GlobalM::StoreOpWithMessage(
                GSOp::PostWithoutMarker(obj),
                || GlobalM::Location(LocationM::Editor(M::SyncPost))
            ),
        }
    }
    async fn attempt_save_async_old(mut post: posts::DataNoMeta, changes: posts::Changed) -> GlobalM {
        const UPDATE_POST_BASE_URL: &str = "/api/posts";
        const SAVE_OLD_MSG: retry::LogPair<'static> = retry::LogPair {
            pre_completion: "saving old post",
            post_completion: "considering changes to post",
        };

        let url = format!("{}/{}", UPDATE_POST_BASE_URL, post.id);
        let req = Request::new(url)
            .method(Method::Patch)
            .json(&changes);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_text_with_retry(
            req,
            &SAVE_OLD_MSG,
            None,
        ).await;
        match res {
            Err(_) => GlobalM::NoOp,
            Ok(_) => {
                if let Some(title) = changes.title {
                    post.title = title;
                }
                if let Some(body) = changes.body {
                    post.body = body;
                }
                GlobalM::StoreOp(GSOp::PostRaw(post))
            }
        }
    }
    pub fn attempt_save(&mut self) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
        // TODO Consider removing the clone here somehow.
        match self {
            Self::New(post) => Some(Box::pin(Self::attempt_save_async_new(post.clone()))),
            Self::Old(post, changes) => Some(Box::pin(Self::attempt_save_async_old(post.clone(), changes.clone()))),
            Self::Undetermined(_) => None,
        }
    }
    async fn attempt_publish_async_new(mut post: posts::NewNoMeta, user_id: uuid::Uuid) -> GlobalM {
        const CREATE_POST_URL: &str = "/api/posts";
        const PUB_NEW_MSG: retry::LogPair<'static> = retry::LogPair {
            pre_completion: "saving and publishing new post",
            post_completion: "parsing created post",
        };
        post.published_at = Some(chrono::Utc::now());
        post.published_by = Some(user_id);
        // save
        let req = Request::new(CREATE_POST_URL)
            .method(Method::Post)
            .json(&post);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_json_with_retry(
            req,
            &PUB_NEW_MSG,
            None,
        ).await;
        match res {
            Err(_) => GlobalM::NoOp,
            Ok(obj) => GlobalM::StoreOpWithAction(GSOp::PostWithoutMarker(obj), |gs: *const GlobalS| {
                let gs = unsafe { gs.as_ref() }.expect("The global state to always exist.");
                gs.post.as_ref().map(|post| GlobalM::ChangePageAndUrl(Location::Viewer(
                    PostMarker::Uuid(post.id).into(),
                )))
                .tap_none(|| log::error!("Post loaded but was not saved."))
                .unwrap_or(GlobalM::NoOp)
            })
        }
    }
    async fn attempt_publish_async_old(post: posts::DataNoMeta, changed: posts::Changed) -> GlobalM {
        const PUB_OLD_MSG: retry::LogPair<'static> = retry::LogPair {
            pre_completion: "saving and publishing old post",
            post_completion: "parsing published post",
        };
        let url = format!("/api/posts/{}/publish", post.id);
        let req = Request::new(url)
            .method(Method::Post);
        let req = if changed.title.is_some() || changed.body.is_some() {
            if let Ok(req) = req.json(&changed) {
                req
            } else {
                return GlobalM::NoOp;
            }
        } else {
            req
        };

        let res = retry::fetch_text_with_retry(
            req, 
            &PUB_OLD_MSG,
            None,
        ).await;
        match res {
            Ok(_) => GlobalM::ChangePageAndUrl(Location::Viewer(
                PostMarker::Uuid(post.id).into(),
            )),
            Err(_) => GlobalM::NoOp,
        }
    }
    pub fn attempt_publish(&mut self, user: &User) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
        match self {
            Self::Undetermined(_) => None,
            Self::New(post) => {
                Some(Box::pin(Self::attempt_publish_async_new(post.clone(), user.id)))
            }
            Self::Old(post, changed) => {
                Some(Box::pin(Self::attempt_publish_async_old(post.clone(), changed.clone())))
            }
        }
    }
}
