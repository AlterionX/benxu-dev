use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::*,
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User as StoreUser,
    },
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

pub fn logout_trigger(_gs: &GlobalS) -> impl GlobalAsyncM {
    use seed::fetch::{Method, Request};
    const LOGOUT_URL: &str = "/api/login";
    Request::new(LOGOUT_URL)
        .method(Method::Delete)
        .fetch_string(|fo| GlobalM::StoreOpWithAction(GSOp::RemoveUser(fo), logout_post_fetch))
}
fn logout_post_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::Grouped(vec![
            GlobalM::UseLoggedOutMenu,
            GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
        ])),
        Failure(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {
    UserName(String),
    Password(String),

    SetCreateMode(bool),

    PasswordConfirmation(String),
    FirstName(String),
    LastName(String),
    Email(String),

    CreateUser,
    CreateCredential,

    CreateSession,

    SetFocus,
}
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn to_url(&self) -> Url {
        Url::new(vec!["blog", "login"])
    }
}
impl S {
    fn create_user_post(&self) -> impl GlobalAsyncM {
        use seed::fetch::{Method, Request};
        const CREATE_USER_URL: &str = "/api/accounts";
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
                                GlobalM::Login(M::CreateCredential),
                                GlobalM::ChangePageAndUrl(Location::Listing(listing::S::default())),
                                GlobalM::UseLoggedInMenu,
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
    fn create_credential_post(&self, u: &StoreUser) -> impl GlobalAsyncM {
        use crate::locations::*;
        use seed::fetch::{Method, Request};
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
    fn create_session_post(&self) -> impl GlobalAsyncM {
        use crate::locations::*;
        use seed::fetch::{Method, Request};
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
                                GlobalM::UseLoggedInMenu,
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

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    // TODO better logging.
    log::debug!("Updating login page with {:?}", m);
    match m {
        // Fields always available, whether signing up or logging in.
        M::UserName(un) => s.username = un,
        M::Password(pw) => s.password = pw,
        // Toggle between signing and logging in.
        M::SetCreateMode(is_create) => s.is_create_mode = is_create,
        // Additional account creation fields.
        M::PasswordConfirmation(pw) => s.password_confirmation = Some(pw),
        M::FirstName(first) => s.first_name = Some(first),
        M::LastName(last) => s.last_name = Some(last),
        M::Email(email) => s.email = Some(email),
        // API calls
        M::CreateUser => {
            log::trace!("Creating a user...");
            orders.perform_g_cmd(s.create_user_post());
        }
        M::CreateSession => {
            log::trace!("Creating a session...");
            orders.perform_g_cmd(s.create_session_post());
        }
        M::CreateCredential => {
            log::trace!("Creating credentials...");
            if let Some(u) = gs.user.as_ref() {
                orders.perform_g_cmd(s.create_credential_post(u));
            }
        }
        M::SetFocus => {
            use wasm_bindgen::JsCast;
            log::trace!("Setting form focus...");
            if let Ok(Some(Ok(node))) = {
                seed::body()
                    .query_selector("input[name=username]")
                    .tap_err(|_| log::error!("Could not find username field!"))
                    .map(|opt_n| {
                        opt_n.map(|n| {
                            (n.dyn_into(): Result<web_sys::HtmlElement, _>)
                                .tap_err(|_| log::error!("Input field is not an HtmlElement!"))
                        })
                    })
            } {
                node.focus()
                    .unwrap_or_else(|_| log::error!("Failed to focuse on the correct form input."));
            }
        }
    }
}
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
                    raw_ev(Ev::Click, move |e| {
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
                        raw_ev(Ev::Click, move |e| {
                            e.prevent_default();
                            M::SetCreateMode(!is_create_mode)
                        }),
                    ],
                ]
            },
        ],
    ]
}
