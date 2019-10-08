use seed::prelude::*;
use serde::{Serialize, Deserialize};

use db_models::posts;
use crate::{
    model::{PostMarker, Store as GlobalS, StoreOperations as GSOp, StoreOpResult as GSOpResult},
    messages::{M as GlobalM, AsyncM as GlobalAsyncM},
    shared,
    locations::Location,
};

pub fn load_post(post_marker: PostMarker) -> impl GlobalAsyncM {
    use seed::fetch::Request;
    const POSTS_URL: &'static str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    Request::new(url).fetch_json(move |fo|
        GlobalM::StoreOpWithAction(
            GSOp::Post(post_marker, fo),
            after_fetch,
        )
    )
}
fn after_fetch(gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    let gs = unsafe { gs.as_ref() }?;
    match (res, &gs.post) {
        (Success, Some(post)) => Some(
            GlobalM::RenderPage(Location::Viewer(S { post_marker: PostMarker::Uuid(post.id.clone()) }))
        ),
        _ => None,
    }
}
pub fn is_restricted_from(gs: &GlobalS) -> bool {
    if let GlobalS { post: Some(post), user, .. } = gs {
        !post.is_published() && user.as_ref().map(|u| !u.can_see_unpublished).unwrap_or(true)
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum M {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct S {
    pub post_marker: PostMarker,
}
impl From<PostMarker> for S {
    fn from(s: PostMarker) -> Self {
        Self { post_marker: s }
    }
}
impl S {
    pub fn to_url(&self) -> Url {
        Url::new(vec![
            "blog",
            "posts",
            self.post_marker.to_string().as_str(),
        ])
    }
}

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}

pub fn render_post(post: &posts::DataNoMeta) -> Node<M> {
        div![
            h1![post.title],
            p![post.body],
        ]
}
pub fn render(s: &S, gs: &GlobalS) -> Node<M> {
    match gs.post.as_ref() {
        Some(post) if s.post_marker.refers_to(post) => render_post(post),
        _ => shared::views::loading(),
    }
}

