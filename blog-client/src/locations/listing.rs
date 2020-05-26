use crate::{
    locations::Location,
    messages::M as GlobalM,
    model::StoreOperations as GSOp,
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
    let res = retry::fetch_json_with_retry(
        url.into(), 
        &POST_LOAD_MSG,
        None,
    ).await;
    match res {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithMessage(GSOp::PostListing(query, obj), || GlobalM::RenderPage(Location::Listing(S { query: None }))),
    }
}
