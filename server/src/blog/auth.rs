use std::{
    sync::Arc,
    marker::PhantomData,
};
use rocket::{
    response::status,
    http::{Status, Cookies},
    request::{
        Request,
        FromRequest,
        Outcome,
    },
    State,
};
use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    crypto::{
        KeyStore,
        CurrAndLastKey,
        token::paseto,
    },
    DefaultAlgo,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum Permission {
    EditPost,
    CreatePost,
    DeletePost,
    Custom { name: String },
}
pub trait Verifiable {
    fn verify(perms: &[Permission]) -> bool;
}
impl Verifiable for () {
    fn verify(_: &[Permission]) -> bool {
        true
    }
}

pub struct CanEdit;
impl CanEdit {
    const REQUIRED_PERMS: [Permission; 1] = [Permission::EditPost];
}
impl Verifiable for CanEdit {
    fn verify(perms: &[Permission]) -> bool {
        for required_perm in Self::REQUIRED_PERMS.iter() {
            let mut found = false;
            for perm in perms.iter() {
                if required_perm == perm {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

pub struct CanDelete;
impl CanDelete {
    const REQUIRED_PERMS: [Permission; 1] = [Permission::DeletePost];
}
impl Verifiable for CanDelete {
    fn verify(perms: &[Permission]) -> bool {
        for required_perm in Self::REQUIRED_PERMS.iter() {
            let mut found = false;
            for perm in perms.iter() {
                if required_perm == perm {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

pub struct CanPost;
impl CanPost {
    const REQUIRED_PERMS: [Permission; 1] = [Permission::CreatePost];
}
impl Verifiable for CanPost {
    fn verify(perms: &[Permission]) -> bool {
        for required_perm in Self::REQUIRED_PERMS.iter() {
            let mut found = false;
            for perm in perms.iter() {
                if required_perm == perm {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

pub struct Admin;
impl Admin {
    const REQUIRED_PERMS: [Permission; 3] = [Permission::EditPost, Permission::CreatePost, Permission::DeletePost];
}
impl Verifiable for Admin {
    fn verify(perms: &[Permission]) -> bool {
        for required_perm in Self::REQUIRED_PERMS.iter() {
            let mut found = false;
            for perm in perms.iter() {
                if required_perm == perm {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

/// L for Level
#[derive(Serialize, Deserialize)]
pub struct Credentials<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    permissions: Vec<Permission>,
    hash: String,
}
impl<L: Verifiable> Credentials<L> {
    const AUTH_COOKIE_NAME: &'static str = "_";
    fn extract(cookies: &Cookies, key: &CurrAndLastKey<paseto::v2::local::Algo>) -> Result<Credentials<()>, ()> {
        let auth_cookie = cookies.get(Self::AUTH_COOKIE_NAME).ok_or(())?;
        let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());

        let token: paseto::token::Data<Credentials<()>, ()> = match paseto::v2::local::Protocol::decrypt(token, &key.curr) {
            Ok(dec) => dec,
            Err(_) => {
                let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
                paseto::v2::local::Protocol::decrypt(token, &key.last).map_err(|_| ())?
            },
        };
        Ok(token.msg)
    }
}
impl<'a, 'r, L: Verifiable> FromRequest<'a, 'r> for Credentials<L> {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let key_store = req
            .guard::<State<Arc<KeyStore<paseto::v2::local::Algo>>>>()
            .map_failure(|_| (Status::InternalServerError, ()))?;
        let key_read_guard = match key_store.curr_and_last() {
            Ok(rg) => rg,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        };

        let cr = match Self::extract(&cookies, &*key_read_guard) {
            Ok(cr) => cr,
            Err(_) => return Outcome::Failure((Status::Forbidden, ())),
        };

        if L::verify(cr.permissions.as_slice()) {
            Outcome::Success(Credentials {
                level: PhantomData,
                permissions: cr.permissions,
                hash: cr.hash,
            })
        } else {
            Outcome::Failure((Status::Forbidden, ()))
        }
    }
}

#[get("/login")]
pub fn get() -> &'static str {
    "a login screen, eventually"
}
/// creates an account
#[post("/login")]
pub fn create() -> status::Custom<()> {
    status::Custom(Status::new(501, "Not yet implemented"), ())
}
/// deletes an account
#[delete("/login")]
pub fn delete() -> status::Custom<()> {
    status::Custom(Status::new(501, "Not yet implemented"), ())
}

