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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M {
    UserName(String),
    Password(String),
    FormSubmission,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    username: String,
    password: String,
}
impl Store {
    fn post(&self) -> impl Future<Item = GlobalM, Error = GlobalM> {
        use seed::fetch::{Request, Method};
        const LOGIN_URL: &'static str = "/api/login";
        let fetch_map = move |fo| GlobalM::DataFetched(LocationWithData::Login(fo));
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
        M::FormSubmission => { orders.skip().perform_cmd(s.post()); },
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
                At::AutoFocus => true;
                At::Type => "password";
            },
            input_ev(Ev::Input, |text| GlobalM::Login(login::M::Password(text))),
        ],
        br![],
        input![
            attrs! { At::Type => "submit" },
            "Login",
        ],
        raw_ev(Ev::Submit, |e| {
            e.prevent_default();
            // TODO actually submit
            GlobalM::Login(login::M::FormSubmission)
        }),
    ]
}
