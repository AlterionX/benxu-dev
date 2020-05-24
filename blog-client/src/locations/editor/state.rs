use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{editor::M, Location, M as LocationM},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User,
    },
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
        Url::new(vec!["blog", "edit", slug])
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
    pub fn attempt_save(&mut self) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
        use seed::fetch::{Method, Request};
        let (url, method) = match self {
            Self::Undetermined(_) => None,
            Self::New(_) => {
                const CREATE_POST_URL: &str = "/api/posts";
                let create_post_method = Method::Post;
                // save
                Some((CREATE_POST_URL.to_owned(), create_post_method))
            }
            Self::Old(post, _) => {
                const UPDATE_POST_BASE_URL: &str = "/api/posts";
                let update_post_method = Method::Patch;
                Some((
                    format!("{}/{}", UPDATE_POST_BASE_URL, post.id),
                    update_post_method,
                ))
            }
        }?;
        let req = Request::new(url).method(method);
        if let Self::New(post) = self {
            // save
            Some(Box::pin(req.send_json(post).fetch_json(move |fo| {
                GlobalM::StoreOpWithAction(GSOp::PostWithoutMarker(fo), |_gs, res| {
                    use crate::model::StoreOpResult::*;
                    match res {
                        Success => {
                            log::debug!(
                                "Post is saved! Modifying state to be `Old` instead of `New`"
                            );
                            Some(GlobalM::Location(LocationM::Editor(M::SyncPost)))
                        }
                        Failure(e) => {
                            log::error!("Post save failed due to {:?}.", e);
                            None
                        }
                    }
                })
            })))
        } else if let Self::Old(post, changes) = self {
            let mut closed_post = post.clone();
            let closed_changes = changes.clone();
            Some(Box::pin(req.send_json(changes).fetch_string_data(
                move |res| match res {
                    Ok(_) => {
                        log::debug!("Launching credential creation");
                        if let Some(title) = closed_changes.title {
                            closed_post.title = title;
                        }
                        if let Some(body) = closed_changes.body {
                            closed_post.body = body;
                        }
                        GlobalM::StoreOp(GSOp::PostRaw(closed_post))
                    }
                    Err(e) => {
                        log::error!("Post save failed due to {:?}.", e);
                        GlobalM::NoOp
                    }
                },
            )))
        } else {
            None
        }
    }
    pub fn attempt_publish(&mut self, user: &User) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
        use seed::fetch::{Method, Request};
        match self {
            Self::Undetermined(_) => None,
            Self::New(post) => {
                const CREATE_POST_URL: &str = "/api/posts";
                post.published_at = Some(chrono::Utc::now());
                post.published_by = Some(user.id);
                // save
                let (url, method) = (CREATE_POST_URL.to_string(), Method::Post);
                let followup = |gs: *const GlobalS, res| {
                    let gs = unsafe { gs.as_ref() }?;
                    if let (GSOpResult::Success, Some(posts::DataNoMeta { id: post_id, .. })) =
                        (res, &gs.post)
                    {
                        Some(GlobalM::ChangePageAndUrl(Location::Viewer(
                            PostMarker::Uuid(*post_id).into(),
                        )))
                    } else {
                        None
                    }
                };
                let reaction =
                    move |fo| GlobalM::StoreOpWithAction(GSOp::PostWithoutMarker(fo), followup);
                let req = Request::new(url)
                    .method(method)
                    .send_json(post)
                    .fetch_json(reaction);
                Some(Box::pin(req))
            }
            Self::Old(post, changed) => {
                let post_id = post.id;
                let (url, method) = (format!("/api/posts/{}/publish", post.id), Method::Post);
                let reaction = move |res| match res {
                    Ok(_) => GlobalM::ChangePageAndUrl(Location::Viewer(
                        PostMarker::Uuid(post_id).into(),
                    )),
                    _ => GlobalM::NoOp,
                };
                let req = Request::new(url).method(method);
                let req = if changed.title.is_some() || changed.body.is_some() {
                    req.send_json(changed)
                } else {
                    req
                };
                let req = req.fetch_string_data(reaction);
                Some(Box::pin(req))
            }
        }
    }
}
