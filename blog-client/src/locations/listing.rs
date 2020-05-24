use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::{Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    requests::PostQuery,
};

mod messages;
mod state;
mod views;
pub use messages::{update, M};
pub use state::S;
pub use views::render;

pub async fn data_load(s: S) -> Result<GlobalM, GlobalM> {
    use seed::browser::service::fetch::Request;
    const POSTS_URL: &str = "/api/posts";
    let query = s.query.unwrap_or_else(PostQuery::default);
    let url = format!("{}?{}", POSTS_URL, query);
    // TODO figure out caching and determining if data already loaded instead of going
    // straight to server all the time.
    Request::new(url)
        .fetch_json(|fo| GlobalM::StoreOpWithAction(GSOp::PostListing(query, fo), after_fetch))
        .await
}
fn after_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::RenderPage(Location::Listing(S { query: None }))),
        Failure(_) => None,
    }
}
