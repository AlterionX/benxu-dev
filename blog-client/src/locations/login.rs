use crate::{
    locations::*,
    messages::{M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp,
    },
    shared::{Authorization, retry},
};
use db_models::models::users;

mod messages;
mod state;
mod views;
pub use messages::{M, update};
pub use state::S;
pub use views::render;

const LOGOUT_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "logging out",
    post_completion: "reading log out response",
};

const FIND_ME_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "discovering myself",
    post_completion: "reading what was discovered",
};

pub async fn logout_trigger() -> GlobalM {
    const LOGOUT_URL: &str = "/api/login";
    let req = Request::new(LOGOUT_URL).method(Method::Delete);
    let res = retry::fetch_process_with_retry(
        LOGOUT_URL.into(),
        &LOGOUT_MSG,
        None,
        |res| res.text(),
    ).await;
    match res {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithAction(GSOp::RemoveUser(obj), logout_post_fetch),
    }
}
fn logout_post_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::Grouped(vec![
            GlobalM::ChangeMenu(Authorization::LoggedOut),
            GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
        ])),
        Failure(_) => None,
    }
}

pub async fn find_current_user() -> Option<users::DataNoMeta> {
    const SELF_URL: &str = "/api/accounts/me";
    log::info!("Detecting if already logged in...");
    let res = retry::fetch_process_with_retry(
        SELF_URL.into(),
        &FIND_ME_MSG,
        Some(1),
        |res| res.json(),
    ).await;
    res.ok()
}


