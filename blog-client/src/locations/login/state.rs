
use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::{Location, M as LocationM, listing, login::{M}},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User as StoreUser,
    },
    shared::Authorization,
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct S {
    pub is_create_mode: bool,
    pub username: String,
    pub password: String,

    pub password_confirmation: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}
impl S {
    pub fn to_url(&self) -> Url {
        Url::new(vec!["blog", "login"])
    }
}
impl S {
    pub fn create_user_post(&self) -> impl GlobalAsyncM {
        use seed::browser::service::fetch::{Method, Request};
        const CREATE_USER_URL: &'static str = "/api/accounts";
        Request::new(CREATE_USER_URL)
            .method(Method::Post)
            .send_json(&users::NewNoMeta {
                user_name: self.username.clone(),
                first_name: self.first_name.clone().unwrap(),
                last_name: self.last_name.clone().unwrap(),
                email: self.email.clone().unwrap(),
            })
            .fetch_json(|fo| {
                GlobalM::StoreOpWithAction(GSOp::User(fo), |_gs, res| {
                    use crate::model::StoreOpResult::*;
                    match res {
                        Success => {
                            log::debug!("Launching credential creation");
                            Some(GlobalM::Grouped(vec![
                                GlobalM::Location(LocationM::Login(M::CreateCredential)),
                                GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
                                GlobalM::ChangeMenu(Authorization::LoggedIn),
                            ]))
                        }
                        Failure(e) => {
                            log::error!("User failed creation due to {:?}.", e);
                            None
                        }
                    }
                })
            })
    }
    pub fn create_credential_post(&self, u: &StoreUser) -> impl GlobalAsyncM {
        use crate::locations::*;
        use seed::browser::service::fetch::{Method, Request};
        const CREDENTIAL_URL: &str = "/api/credentials/pws";
        Request::new(CREDENTIAL_URL)
            .method(Method::Post)
            .send_json(&CreatePassword {
                user_id: u.id,
                password: self.password.clone(),
            })
            .fetch(|fo| {
                if fo.response().is_ok() {
                    GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default()))
                } else {
                    GlobalM::NoOp
                }
            })
    }
    pub fn create_session_post(&self) -> impl GlobalAsyncM {
        use crate::locations::*;
        use seed::browser::service::fetch::{Method, Request};
        const LOGIN_URL: &str = "/api/login";
        Request::new(LOGIN_URL)
            .method(Method::Post)
            .send_json(&Authentication::Password(Password {
                user_name: self.username.clone(),
                password: self.password.clone(),
            }))
            .fetch_json(move |fo| {
                GlobalM::StoreOpWithAction(GSOp::User(fo), |_gs, res| {
                    use crate::model::StoreOpResult::*;
                    match res {
                        Success => {
                            log::trace!("Logged in. Redirect to homepage.");
                            Some(GlobalM::Grouped(vec![
                                GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
                                GlobalM::ChangeMenu(Authorization::LoggedIn),
                            ]))
                        }
                        Failure(e) => {
                            log::trace!("Attempt to create session failed with {:?} error.", e);
                            None
                        }
                    }
                })
            })
    }
}