use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::{Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    requests::PostQuery,
    shared::retry,
};

mod messages;
mod state;
mod views;
pub use messages::{update, M};
pub use state::S;
pub use views::render;

pub async fn data_load(s: S) -> GlobalM {
    const POST_LOAD_MSG: retry::LogPair<'static> = retry::LogPair {
        pre_completion: "fetching posts",
        post_completion: "parsing fetched posts",
    };
    const POSTS_URL: &str = "/api/posts";
    let query = s.query.unwrap_or_else(PostQuery::default);
    let url = format!("{}?{}", POSTS_URL, query);
    let res = retry::fetch_process_with_retry(
        url.into(), 
        &POST_LOAD_MSG,
        None,
        |res| res.json()
    ).await;
    match res {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithAction(GSOp::PostListing(query, obj), after_fetch),
    }
}
fn after_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::RenderPage(Location::Listing(S { query: None }))),
        Failure(_) => None,
    }
}
