
use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{Location, M as LocationM, listing, login::{M}},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        StoreOperations as GSOp, User as StoreUser,
    },
    shared::{Authorization, retry},
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

const CREATE_USER_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "creating user",
    post_completion: "parsing created user",
};
const CREATE_CREDENTIAL_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "creating credential",
    post_completion: "parsing created credential",
};
const CREATE_SESSION_MSG: retry::LogPair<'static> = retry::LogPair {
    pre_completion: "creating session",
    post_completion: "parsing created session",
};

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
        Url::new().set_path(&["blog", "login"])
    }
}
impl S {
    pub fn create_user_post(&self) -> impl GlobalAsyncM {
        let data = users::NewNoMeta {
            user_name: self.username.clone(),
            first_name: self.first_name.clone().unwrap(),
            last_name: self.last_name.clone().unwrap(),
            email: self.email.clone().unwrap(),
        };
        Self::create_user_post_async(data)
    }

    async fn create_user_post_async<'a>(data: users::NewNoMeta) -> GlobalM {
        const CREATE_USER_URL: &'static str = "/api/accounts";
        let req = Request::new(CREATE_USER_URL)
            .method(Method::Post)
            .json(&data);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_process_with_retry(
            req,
            &CREATE_CREDENTIAL_MSG,
            None,
            |res| res.json(),
        ).await;
        match res {
            Err(_) => GlobalM::NoOp,
            Ok(obj) =>
                GlobalM::StoreOpWithAction(GSOp::User(obj), |_gs, res| {
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
        }
    }

    pub fn create_credential_post(&self, u: &StoreUser) -> impl GlobalAsyncM {
        let pw = CreatePassword {
            user_id: u.id,
            password: self.password.clone(),
        };
        Self::create_credential_post_async(pw)
    }

    async fn create_credential_post_async(pw: CreatePassword) -> GlobalM {
        const CREDENTIAL_URL: &str = "/api/credentials/pws";
        let req = Request::new(CREDENTIAL_URL)
            .method(Method::Post)
            .json(&pw);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_process_with_retry(
            req,
            &CREATE_USER_MSG,
            None,
            |res| async { Ok(()) },
        ).await;
        match res {
            Err(_) => GlobalM::NoOp,
            Ok(_) => GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
        }
    }

    pub fn create_session_post(&self) -> impl GlobalAsyncM {
        let auth = Authentication::Password(Password {
            user_name: self.username.clone(),
            password: self.password.clone(),
        });
        Self::create_session_post_async(auth)
    }

    async fn create_session_post_async(auth: Authentication) -> GlobalM {
        use crate::locations::*;
        const LOGIN_URL: &str = "/api/login";
        let req = Request::new(LOGIN_URL)
            .method(Method::Post)
            .json(&auth);
        let req = if let Ok(req) = req {
            req
        } else {
            return GlobalM::NoOp;
        };
        let res = retry::fetch_process_with_retry(
            req,
            &CREATE_SESSION_MSG,
            None,
            |res| res.json(),
        ).await;
        match res {
            // TODO Display error message.
            Err(_) => GlobalM::NoOp,
            Ok(obj) => GlobalM::StoreOpWithAction(GSOp::User(obj), |_gs, res| {
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
        }
    }
}