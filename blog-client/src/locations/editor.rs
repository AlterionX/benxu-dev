use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::Location,
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User,
    },
};
use db_models::models::*;

pub async fn load_post(post_marker: PostMarker) -> Result<GlobalM, GlobalM> {
    const POSTS_URL: &str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    use seed::fetch::Request;
    Request::new(url)
        .fetch_json(move |fo| {
            GlobalM::StoreOpWithAction(GSOp::Post(post_marker, fo), |gs, res| {
                use GSOpResult::*;
                let gs = unsafe { gs.as_ref() }?;
                match (res, &gs.post) {
                    (Success, Some(post)) => Some(GlobalM::RenderPage(Location::Editor(S::Old(
                        post.clone(),
                        posts::Changed::default(),
                    )))),
                    _ => None,
                }
            })
        })
        .await
}
pub fn is_restricted_from(s: &S, gs: &GlobalS) -> bool {
    if let GlobalS {
        user: Some(user), ..
    } = gs
    {
        // TODO move this check onto the server for security
        match s {
            S::Old(stored_post, _) => !stored_post.is_published() && !user.can_see_unpublished,
            S::New(_) => false,
            S::Undetermined(_) => false,
        }
    } else {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {
    Title(String),
    Body(String),
    Slug(String),
    Publish,
    Save,

    SyncPost,
}
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
    fn update_title(&mut self, title: String) {
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
    fn update_body(&mut self, body: String) {
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
    fn update_slug(&mut self, slug: String) {
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
    fn attempt_save(&mut self) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
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
                            Some(GlobalM::Editor(M::SyncPost))
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
    fn attempt_publish(&mut self, user: &User) -> Option<std::pin::Pin<Box<dyn GlobalAsyncM>>> {
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

fn update_post(to_update: &mut posts::DataNoMeta, updated: &posts::DataNoMeta) {
    to_update.created_by = updated.created_by;
    to_update.created_at = updated.created_at;
    to_update.published_by = updated.published_by;
    to_update.published_at = updated.published_at;
    to_update.archived_by = updated.archived_by;
    to_update.archived_at = updated.archived_at;
    to_update.deleted_by = updated.deleted_by;
    to_update.deleted_at = updated.deleted_at;
    to_update.title = updated.title.clone();
    to_update.body = updated.body.clone();
    to_update.slug = updated.slug.clone();
}
pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    use M::*;
    match s {
        S::Undetermined(_) => return,
        _ => (),
    };
    match m {
        Title(title) => s.update_title(title),
        Body(body) => s.update_body(body),
        Slug(slug) => s.update_slug(slug),
        Publish => {
            let _ = (|| {
                let user = gs.user.as_ref()?;
                let req = s.attempt_publish(user)?;
                orders.perform_g_cmd(req);
                Some(())
            })();
        }
        Save => {
            if let Some(req) = s.attempt_save() {
                orders.perform_g_cmd(req);
            }
        }

        SyncPost => {
            if let Some(updated) = &gs.post {
                match s {
                    S::Old(post, _) if post.id == updated.id => update_post(post, updated),
                    _ => {
                        orders.send_g_msg(GlobalM::ChangePageAndUrl(Location::Editor(S::Old(
                            updated.clone(),
                            posts::Changed::default(),
                        ))));
                    }
                }
            }
        }
    }
}

pub use views::render;
mod views {
    use seed::prelude::*;

    use super::{M, S};
    use crate::model::Store as GlobalS;

    pub fn render(s: &S, _gs: &GlobalS) -> Vec<Node<M>> {
        vec![
            heading(),
            editor(s).unwrap_or_else(crate::shared::views::loading),
        ]
    }

    pub fn heading() -> Node<M> {
        h1![attrs! { At::Class => "as-h3" }, "Editing"]
    }

    fn get_title_slug_body(s: &S) -> Option<(&str, Option<&str>, &str)> {
        let (t, slug, b) = match s {
            S::New(post) => (&post.title, post.slug.as_ref(), &post.body),
            S::Old(post, changed) => (
                changed.title.as_ref().unwrap_or(&post.title),
                post.slug.as_ref(),
                changed.body.as_ref().unwrap_or(&post.body),
            ),
            _ => return None,
        };
        let slug = slug.map(String::as_str);
        Some((t, slug, b))
    }
    fn title_field(title: &str) -> Node<M> {
        div![
            attrs! { At::Class => "editor-title" },
            input![
                {
                    let mut attrs = attrs! {
                        At::Placeholder => "Title";
                        At::Type => "text";
                        At::Name => "title",
                        At::Value => title,
                    };
                    attrs.add_multiple(At::Class, &["single-line-text-entry", "as-h1"]);
                    attrs
                },
                input_ev(Ev::Input, M::Title),
            ],
        ]
    }
    fn slug_field(slug: &str, hint: &str) -> Node<M> {
        div![
            attrs! { At::Class => "editor-slug" },
            label![
                {
                    let mut attrs = attrs! {
                        At::For => "slug",
                    };
                    attrs.add_multiple(At::Class, &["same-line-label", "as-pre"]);
                    attrs
                },
                "/blog/posts/",
            ],
            input![
                {
                    let mut attrs = attrs! {
                        At::Placeholder => hint;
                        At::Type => "text";
                        At::Name => "slug",
                        At::Value => slug,
                    };
                    attrs.add_multiple(At::Class, &["single-line-text-entry", "as-pre"]);
                    attrs
                },
                input_ev(Ev::Input, M::Slug),
            ],
        ]
    }
    fn body_field(body: &str) -> Node<M> {
        div![
            attrs! {
                At::Class => "editor-body",
            },
            textarea![
                {
                    let mut attrs = attrs! {
                        At::Placeholder => "Write your post here!";
                        At::Type => "text";
                        At::Name => "body",
                    };
                    attrs.add_multiple(At::Class, &["multi-line-text-entry"]);
                    attrs
                },
                body,
                input_ev(Ev::Input, M::Body),
            ],
        ]
    }
    fn action_buttons(s: &S) -> Node<M> {
        div![
            attrs! {
                At::Class => "editor-actions",
            },
            input![
                attrs! {
                    At::Class => "inline-button",
                    At::Type => "submit",
                    At::Value => "Save",
                },
                ev(Ev::Click, |e| {
                    e.prevent_default();
                    M::Save
                }),
            ],
            if s.is_publishable() {
                input![
                    attrs! {
                        At::Class => "inline-button",
                        At::Type => "submit",
                        At::Value => "Publish",
                    },
                    ev(Ev::Click, |e| {
                        e.prevent_default();
                        M::Publish
                    }),
                ]
            } else {
                empty![]
            },
        ]
    }
    pub fn editor(s: &S) -> Option<Node<M>> {
        let (title, slug, body) = get_title_slug_body(s)?;
        let slug_hint_mem = slug
            .map(|_| None)
            .unwrap_or_else(|| s.old_ref().map(|p| p.id.to_hyphenated_ref().to_string()));
        let slug_hint = slug_hint_mem
            .as_ref()
            .map(String::as_str)
            .or(slug)
            .unwrap_or("");
        Some(div![
            attrs! { At::Class => "editor" },
            title_field(title),
            slug_field(slug.unwrap_or(""), slug_hint),
            body_field(body),
            action_buttons(s),
        ])
    }
}
