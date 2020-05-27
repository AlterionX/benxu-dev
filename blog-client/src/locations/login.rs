use crate::{
    locations::*,
    messages::{M as GlobalM},
    model::StoreOperations as GSOp,
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
    let res = retry::fetch_text_with_retry(
        req,
        &LOGOUT_MSG,
        None,
    ).await;
    match res {
        Err(_) => GlobalM::NoOp,
        Ok(obj) => GlobalM::StoreOpWithMessage(GSOp::RemoveUser(obj), || GlobalM::Grouped(vec![
            GlobalM::ChangeMenu(Authorization::LoggedOut),
            GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
        ])),
    }
}

pub async fn find_current_user() -> Option<users::DataNoMeta> {
    const SELF_URL: &str = "/api/accounts/me";
    log::info!("Detecting if already logged in...");
    let res = retry::fetch_json_with_retry(
        SELF_URL.into(),
        &FIND_ME_MSG,
        Some(1),
    ).await;
    res.ok()
}


