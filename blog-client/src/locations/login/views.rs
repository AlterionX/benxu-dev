
use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::{*, login::{M, S}},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User as StoreUser,
    },
    shared::Authorization,
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

pub fn render(s: &S, _gs: &GlobalS) -> Node<M> {
    div![
        attrs! {
            At::Class => "login-wrapper",
        },
        form![
            div![
                label![attrs! { At::For => "username" }, "Username",],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "username";
                        At::AutoFocus => true;
                        At::Type => "text";
                        At::Name => "username";
                    },
                    input_ev(Ev::Input, |text| {
                        log::debug!("Updating username to {:?}!", text);
                        M::UserName(text)
                    }),
                ],
            ],
            div![
                label![attrs! { At::For => "password" }, "Password",],
                input![
                    attrs! {
                        At::Class => "single-line-text-entry";
                        At::Placeholder => "password";
                        At::Type => "password";
                        At::Name => "password";
                    },
                    input_ev(Ev::Input, M::Password),
                ],
            ],
            if s.is_create_mode {
                vec![
                    div![
                        label![
                            attrs! { At::For => "password_confirmation" },
                            "Confirm password",
                        ],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "password";
                                At::Type => "password";
                                At::Name => "password_confirmation";
                            },
                            input_ev(Ev::Input, M::PasswordConfirmation),
                        ],
                    ],
                    div![
                        label![attrs! { At::For => "first_name" }, "First name",],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "First Name";
                                At::Type => "text";
                                At::Name => "first_name";
                            },
                            input_ev(Ev::Input, M::FirstName),
                        ],
                    ],
                    div![
                        label![attrs! { At::For => "last_name" }, "Last name",],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "last name";
                                At::Type => "text";
                                At::Name => "last_name";
                            },
                            input_ev(Ev::Input, M::LastName),
                        ],
                    ],
                    div![
                        label![attrs! { At::For => "email" }, "Please enter your email.",],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "email";
                                At::Type => "email";
                                At::Name => "email";
                            },
                            input_ev(Ev::Input, M::Email),
                        ],
                    ],
                ]
            } else {
                vec![]
            },
            {
                let is_create_mode = s.is_create_mode;
                div![input![
                    attrs! {
                        At::Type => "submit",
                        At::Value => if is_create_mode { "Sign up" } else { "Sign in" },
                    },
                    ev(Ev::Click, move |e| {
                        e.prevent_default();
                        if is_create_mode {
                            M::CreateUser
                        } else {
                            M::CreateSession
                        }
                    }),
                ],]
            },
            {
                let is_create_mode = s.is_create_mode;
                div![
                    p![
                        attrs! {
                            At::Class => "same-line-label",
                        },
                        if s.is_create_mode {
                            "Already have an account?"
                        } else {
                            "Don't have an account?"
                        }
                    ],
                    button![
                        if is_create_mode { "Sign in" } else { "Sign up" },
                        ev(Ev::Click, move |e| {
                            e.prevent_default();
                            M::SetCreateMode(!is_create_mode)
                        }),
                    ],
                ]
            },
        ],
    ]
}