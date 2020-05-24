use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::*,
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User as StoreUser,
    },
    shared::Authorization,
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

mod messages;
mod state;
mod views;
pub use messages::{M, update};
pub use state::S;
pub use views::render;

pub async fn logout_trigger() -> Result<GlobalM, GlobalM> {
    use seed::fetch::{Method, Request};
    const LOGOUT_URL: &str = "/api/login";
    Request::new(LOGOUT_URL)
        .method(Method::Delete)
        .fetch_string(|fo| GlobalM::StoreOpWithAction(GSOp::RemoveUser(fo), logout_post_fetch))
        .await
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

pub async fn find_current_user() -> seed::fetch::FetchObject<users::DataNoMeta> {
    const SELF_URL: &str = "/api/accounts/me";
    log::info!("Detecting if already logged in...");
    seed::Request::new(SELF_URL)
        .fetch_json(|f: seed::fetch::FetchObject<db_models::users::DataNoMeta>| f)
        .await
        .unwrap_or_else(|e| e)
}


