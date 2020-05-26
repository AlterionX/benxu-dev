use crate::{
    locations::{Location},
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared::retry,
};

mod messages;
mod state;
mod views;
pub use messages::{M, update};
pub use state::S;
pub use views::render;

const POST_LOAD_MSGS: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "loading post",
    post_completion: "parsing loaded post",
};

pub async fn load_post(post_marker: PostMarker) -> GlobalM {
    const POSTS_URL: &str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    let fo = retry::fetch_process_with_retry(
        url.into(),
        &POST_LOAD_MSGS,
        None,
        |res| res.json(),
    ).await;
    match fo {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithAction(GSOp::Post(post_marker, obj), after_fetch),
    }
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
