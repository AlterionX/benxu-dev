use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{Location},
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared,
};
use db_models::posts;

mod messages;
mod state;
mod views;
pub use messages::{M, update};
pub use state::S;
pub use views::render;

pub async fn load_post(post_marker: PostMarker) -> Result<GlobalM, GlobalM> {
    use seed::browser::service::fetch::Request;
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
