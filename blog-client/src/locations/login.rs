use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use login_enum::{Authentication, Password};
use db_models::models::users;
use crate::{
    messages::M as GlobalM,
    model::{Store as GlobalS, StoreOperations as GSOp},
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

    CreateUser,
    CreateCredential,

    CreateSession,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct S {
    is_create_mode: bool,
    username: String,
    password: String,

    password_confirmation: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
}
impl S {
    fn create_user_post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const CREATE_USER_URL: &'static str = "/api/accounts";
        let fetch_map = move |fo| GlobalM::StoreOp(
            GSOp::UpdateUser(fo),
            |res| res.ok().map(|_| GlobalM::Login(M::CreateCredential))
        );
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
    fn create_credential_post(&self) -> impl Future<Item = M, Error = M> {
        use seed::fetch::{Request, Method};
        const CREDENTIAL_URL: &'static str = "/api/credentials/pws";
        let fetch_map = move |_: FetchObject<()>| M::CreateCredential;
        Request::new(CREDENTIAL_URL)
            .method(Method::Post)
            .send_json(&Authentication::Password(Password {
                user_name: self.username.clone(),
                password: self.password.clone(),
            }))
            .fetch_json(fetch_map)
    }
    fn create_session_post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const LOGIN_URL: &'static str = "/api/login";
        let fetch_map = move |fo| GlobalM::StoreOp(
            GSOp::UpdateUser(fo),
            |_| None,
        );
        Request::new(LOGIN_URL)
            .method(Method::Post)
            .send_json(&Authentication::Password(Password {
                user_name: self.username.clone(),
                password: self.password.clone(),
            }))
            .fetch_json(fetch_map)
    }
}

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    crate::log(format!("Updating store with {:?}", m).as_str());
    match m {
        M::UserName(un) => s.username = un,
        M::Password(pw) => s.password = pw,

        M::ToggleCreateMode => { update(M::SetCreateMode(!s.is_create_mode), s, gs, orders) },
        M::SetCreateMode(is_mode) => s.is_create_mode = is_mode,

        M::PasswordConfirmation(pw) => s.password_confirmation = Some(pw),
        M::FirstName(first) => s.first_name = Some(first),
        M::LastName(last) => s.last_name = Some(last),
        M::Email(email) => s.email = Some(email),

        M::CreateUser => { orders.perform_g_cmd(s.create_user_post()); },
        M::CreateSession => { orders.perform_g_cmd(s.create_session_post()); },
        M::CreateCredential => { orders.perform_cmd(s.create_credential_post()); },
    }
}

pub fn render(s: &S, _gs: &GlobalS) -> Node<M> {
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
                M::UserName(text)
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
            input_ev(Ev::Input, |text| M::Password(text)),
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
                    input_ev(Ev::Input, |text| M::PasswordConfirmation(text)),
                ],
                br![],
                p!["Please enter your first name."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "First Name";
                        At::Type => "text";
                    },
                    input_ev(Ev::Input, |text| M::FirstName(text)),
                ],
                br![],
                p!["Please enter your last name."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "last name";
                        At::Type => "text";
                    },
                    input_ev(Ev::Input, |text| M::LastName(text)),
                ],
                br![],
                p!["Please enter your email."],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "email";
                        At::Type => "email";
                    },
                    input_ev(Ev::Input, |text| M::Email(text)),
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
                        M::CreateUser
                    }),
                ],
                br![],
                p!["Already have an account?"],
                button![
                    "Log in",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        M::SetCreateMode(false)
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
                        M::CreateSession
                    }),
                ],
                br![],
                p!["Don't have an account?"],
                button![
                    "Sign up",
                    raw_ev(Ev::Click, |e| {
                        e.prevent_default();
                        M::SetCreateMode(true)
                    }),
                ],
            ]
        },
    ]
}

