use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use login_enum::*;
use db_models::models::posts;
use crate::{
    messages::M as GlobalM,
    locations::*,
};

#[derive(Debug, Clone)]
pub enum M {
    UserName(String),
    Password(String),

    ToggleCreateMode,
    SetCreateMode(bool),

    PasswordConfirmation(String),
    FirstName(String),
    LastName(String),
    Email(String),

    Creation,
    Login,

    UserCreation(FetchObject<users::DataNoMeta>),
    CredentialCreation(FetchObject<users::DataNoMeta>),
    QueryResult(FetchObject<users::DataNoMeta>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    is_create_mode: bool,
    username: String,
    password: String,

    password_confirmation: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
}
impl Store {
    fn create_user_post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const CREATE_USER_URL: &'static str = "/api/accounts";
        let fetch_map = move |fo| GlobalM::Login(M::UserCreation(fo));
        Request::new(CREATE_USER_URL)
            .method(Method::Post)
            .send_json(&users::NewNoMeta {
                user_name: self.username.clone(),
                first_name: self.first_name.clone().unwrap(),
                last_name: self.last_name.clone().unwrap(),
                email: self.email.clone().unwrap(),
            })
            .fetch_json(fetch_map)
    }
    fn create_credential_post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const LOGIN_URL: &'static str = "/api/credentials/pws";
        let fetch_map = move |fo| GlobalM::Login(M::CredentialCreation(fo));
        Request::new(LOGIN_URL)
            .method(Method::Post)
            .send_json(&Authentication::Password(Password {
                user_name: self.username.clone(),
                password: self.password.clone(),
            }))
            .fetch_json(fetch_map)
    }
    fn post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const LOGIN_URL: &'static str = "/api/login";
        let fetch_map = move |fo| GlobalM::Login(M::QueryResult(fo));
        Request::new(LOGIN_URL)
            .method(Method::Post)
            .send_json(&Authentication::Password(Password {
                user_name: self.username.clone(),
                password: self.password.clone(),
            }))
            .fetch_json(fetch_map)
    }
}

pub fn update(m: M, s: &mut Store, gs: &mut GlobalStore, orders: &mut impl Orders<GlobalM>) {
    crate::log(format!("Updating store with {:?}", m).as_str());
    match m {
        M::UserName(un) => s.username = un,
        M::Password(pw) => s.password = pw,

        M::ToggleCreateMode => update(M::SetCreateMode(!s.is_create_mode), s, gs, orders),
        M::SetCreateMode(is_mode) => s.is_create_mode = is_mode,

        M::PasswordConfirmation(pw) => s.password_confirmation = Some(pw),
        M::FirstName(first) => s.first_name = Some(first),
        M::LastName(last) => s.last_name = Some(last),
        M::Email(email) => s.email = Some(email),

        M::Creation => {
            orders
                .skip()
                .perform_cmd(s.create_user_post());
        },
        M::Login => {
            orders
                .skip()
                .perform_cmd(s.post());
        },
        M::UserCreation(fo) => {
            match fo.response() {
                Err(e) => crate::log(format!("Error {:?} occurred! TODO: show an error.", e).as_str()),
                Ok(fetched) => {
                    let unparsed = fetched.data;
                    let parsed = crate::model::User {
                        id: unparsed.id,
                        name: crate::model::Name {
                            first: unparsed.first_name.unwrap_or("unknown".to_owned()),
                            last: unparsed.last_name.unwrap_or("unknown".to_owned()),
                            nickname: "unknown".to_owned(),
                        },
                        can_see_unpublished: false,
                    };
                    gs.user.replace(parsed);
                    orders.perform_cmd(s.create_credential_post());
                },
            }
        },
        M::QueryResult(fo) => {
            match fo.response() {
                Err(e) => crate::log(format!("Error {:?} occurred! TODO: show an error.", e).as_str()),
                Ok(fetched) => {
                    let unparsed = fetched.data;
                    let parsed = crate::model::User {
                        id: unparsed.id,
                        name: crate::model::Name {
                            first: unparsed.first_name.unwrap_or("unknown".to_owned()),
                            last: unparsed.last_name.unwrap_or("unknown".to_owned()),
                            nickname: "unknown".to_owned(),
                        },
                        can_see_unpublished: false,
                    };
                    gs.user.replace(parsed);
                },
            }
        },
        // TODO consider if there are any updates for when credentials get created.
        M::CredentialCreation(_fo) => (),
    }
}
pub fn render(s: &Store, gs: &GlobalStore) -> seed::dom_types::Node<GlobalM> {
    form![
        p!["Please enter your username"],
        input![
            attrs! {
                At::Class => "single-line-text-entry";
                At::Placeholder => "username";
                At::AutoFocus => true;
                At::Type => "text";
            },
            input_ev(Ev::Input, |text| {
                crate::log(format!("Updating username to {:?}!", text).as_str());
                GlobalM::Login(login::M::UserName(text))
            }),
        ],
        br![],
        p!["Please enter your password"],
        input![
            attrs! {
                At::Class => "single-line-text-entry";
                At::Placeholder => "password";
                At::Type => "password";
            },
            input_ev(Ev::Input, |text| GlobalM::Login(login::M::Password(text))),
        ],
        br![],
        if s.is_create_mode {
            vec![
                p!["Please confirm your password."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "password";
                        At::Type => "password";
                    },
                    input_ev(Ev::Input, |text| GlobalM::Login(login::M::PasswordConfirmation(text))),
                ],
                br![],
                p!["Please enter your first name."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "First Name";
                        At::Type => "text";
                    },
                    input_ev(Ev::Input, |text| GlobalM::Login(login::M::FirstName(text))),
                ],
                br![],
                p!["Please enter your last name."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "last name";
                        At::Type => "text";
                    },
                    input_ev(Ev::Input, |text| GlobalM::Login(login::M::LastName(text))),
                ],
                br![],
                p!["Please enter your email."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "email";
                        At::Type => "email";
                    },
                    input_ev(Ev::Input, |text| GlobalM::Login(login::M::Email(text))),
                ],
                br![],
            ]
        } else { vec![] },
        if s.is_create_mode {
            vec![
                input![
                    attrs! { At::Type => "submit" },
                    "Already have an account? Login instead!",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        GlobalM::Login(M::Creation)
                    }),
                ],
                br![],
                p!["Already have an account?"],
                button![
                    "Log in",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        GlobalM::Login(M::SetCreateMode(false))
                    }),
                ],
            ]
        } else {
            vec![
                input![
                    attrs! { At::Type => "submit" },
                    "Don't have an account yet? Sign up.",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        GlobalM::Login(M::Login)
                    }),
                ],
                br![],
                p!["Don't have an account?"],
                button![
                    "Sign up",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        GlobalM::Login(M::SetCreateMode(true))
                    }),
                ],
            ]
        },
    ]
}

