
use seed::prelude::*;
use serde::{Deserialize, Serialize};
use tap::*;

use crate::{
    locations::{Location, M as LocationM, listing, login::{S}},
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{
        Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp, User as StoreUser,
    },
    shared::Authorization,
};
use db_models::models::users;
use login_enum::{Authentication, CreatePassword, Password};

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
            let _ = (|| {
                use wasm_bindgen::JsCast;
                log::trace!("Setting form focus...");
                let el: web_sys::HtmlElement = seed::body()
                    .query_selector("input[name=username]")
                    .tap_err(|_| log::error!("Could not find username field!"))
                    .ok()??
                    .dyn_into()
                    .tap_err(|_| log::error!("Input field is not an HtmlElement!"))
                    .ok()?;
                el.focus()
                    .tap_err(|_| log::error!("Failed to focus on the username form input."))
                    .ok()?;
                Some(())
            })();
        }
    }
}