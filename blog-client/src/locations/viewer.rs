use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared,
};
use db_models::posts;

pub async fn load_post(post_marker: PostMarker) -> Result<GlobalM, GlobalM> {
    use seed::fetch::Request;
    const POSTS_URL: &str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    Request::new(url)
        .fetch_json(move |fo| GlobalM::StoreOpWithAction(GSOp::Post(post_marker, fo), after_fetch))
        .await
}
fn after_fetch(gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    let gs = unsafe { gs.as_ref() }?;
    match (res, &gs.post) {
        (Success, Some(post)) => Some(GlobalM::RenderPage(Location::Viewer(S {
            post_marker: PostMarker::Uuid(post.id),
        }))),
        _ => None,
    }
}
pub fn is_restricted_from(gs: &GlobalS) -> bool {
    if let GlobalS {
        post: Some(post),
        user,
        ..
    } = gs
    {
        !post.is_published()
            && user
                .as_ref()
                .map(|u| !u.can_see_unpublished)
                .unwrap_or(true)
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        Url::new(vec!["blog", "posts", self.post_marker.to_string().as_str()])
    }
}

pub fn update(m: M, _s: &mut S, _gs: &GlobalS, _orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}

pub fn render_post(post: &posts::DataNoMeta) -> Node<M> {

    div![
        attrs! { At::Class => "post" },
        h1![post.title],
        md![post.body.as_str()],
    ]
}
pub fn render(s: &S, gs: &GlobalS) -> Node<M> {
    match gs.post.as_ref() {
        Some(post) if s.post_marker.refers_to(post) => render_post(post),
        _ => shared::views::loading(),
    }
}
