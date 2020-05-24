use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
};
use db_models::models::*;

mod messages;
mod state;
mod views;
pub use messages::{update, M};
pub use state::S;
pub use views::render;

pub async fn load_post(post_marker: PostMarker) -> Result<GlobalM, GlobalM> {
    const POSTS_URL: &str = "/api/posts";
    let url = format!("{}/{}", POSTS_URL, post_marker);
    use seed::browser::service::fetch::Request;
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
