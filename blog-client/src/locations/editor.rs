use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared::retry,
};
use db_models::models::*;

mod messages;
mod state;
mod views;
pub use messages::{update, M};
pub use state::S;
pub use views::render;

const POST_LOAD_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "loading editor post",
    post_completion: "parsing loaded editor post",
};

pub async fn load_post(post_marker: PostMarker) -> GlobalM {
    const POSTS_URL: &str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    let fo = retry::fetch_process_with_retry(
        url.into(),
        &POST_LOAD_MSG,
        None,
        |res| res.json(),
    ).await;
    match fo {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithAction(GSOp::Post(post_marker, obj), |gs, res| {
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
    }
}
pub fn is_restricted_from(s: &S, gs: &GlobalS) -> bool {
    if let Some(user) = gs.user.as_ref() {
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
