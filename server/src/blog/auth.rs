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
    Deserializer,
    Deserialize,
};
use crate::{
    crypto::{
        KeyStore,
        CurrAndLastKey,
        token::paseto,
    },
    blog::db as db,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum Permission {
    EditPost,
    CreatePost,
    DeletePost,
    Custom { name: String },
}
pub struct PermissionVec(Vec<Permission>);

pub trait Verifiable {
    fn verify(perms: &[Permission]) -> bool;
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

pub type AnyPermissions = ();
impl Verifiable for AnyPermissions {
    fn verify(_: &[Permission]) -> bool {
        true
    }
}

/// L for Level
#[derive(Serialize)]
pub struct Credentials<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    permissions: Vec<Permission>,
    hash: String,
}
impl<L: Verifiable> Credentials<L> {
    const AUTH_COOKIE_NAME: &'static str = "_";
    fn extract(cookies: &Cookies, key: &CurrAndLastKey<paseto::v2::local::Algo>) -> Result<Credentials<AnyPermissions>, ()> {
        let auth_cookie = cookies.get(Self::AUTH_COOKIE_NAME).ok_or(())?;
        let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());

        let token: paseto::token::Data<Credentials<AnyPermissions>, ()> = match paseto::v2::local::Protocol::decrypt(token, &key.curr) {
            Ok(dec) => dec,
            Err(_) => {
                let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
                paseto::v2::local::Protocol::decrypt(token, &key.last).map_err(|_| ())?
            },
        };
        Ok(token.msg)
    }
}
impl <'de> Deserialize<'de> for Credentials<AnyPermissions> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        enum Field {
            Permissions,
            Hash,
            Ignore,
        }
        struct FieldVisitor;
        impl <'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut serde::export::Formatter) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "field identifier")
            }
            fn visit_u64<E>(self, value: u64) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    0u64 => serde::export::Ok(Field::Permissions),
                    1u64 => serde::export::Ok(Field::Hash),
                    _ => serde::export::Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(value),
                            &"field index 0 <= i < 2"
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    "permissions" => serde::export::Ok(Field::Permissions),
                    "hash" => serde::export::Ok(Field::Hash),
                    _ => serde::export::Ok(Field::Ignore)
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    b"permissions" => serde::export::Ok(Field::Permissions),
                    b"hash" => serde::export::Ok(Field::Hash),
                    _ => serde::export::Ok(Field::Ignore)
                }
            }
        }
        impl <'de> serde::Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> serde::export::Result<Self, D::Error> where D: serde::Deserializer<'de> {
                serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct Visitor<'de> {
            lifetime: serde::export::PhantomData<&'de ()>,
        }
        impl <'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Credentials<AnyPermissions>;
            fn expecting(&self, formatter: &mut serde::export::Formatter) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "struct Credentials")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de> {
                let permissions = serde::de::SeqAccess::next_element::<Vec<Permission>>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(0usize, &"struct Credentials with 2 elements")
                )?;
                let hash = serde::de::SeqAccess::next_element::<String>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(1usize, &"struct Credentials with 2 elements")
                )?;
                Ok(
                    Credentials{
                        level: PhantomData,
                        permissions: permissions,
                        hash: hash,
                    }
                )
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> serde::export::Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
                let mut permissions = None;
                let mut hash = None;
                while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Permissions => permissions = if permissions.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("permissions"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<Vec<Permission>>(&mut map)?)
                        },
                        Field::Hash => hash = if hash.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("hash"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<String>(&mut map)?)
                        },
                        _ => { let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(&mut map)?; },
                    }
                }
                let permissions = match permissions {
                    serde::export::Some(permissions) => permissions,
                    serde::export::None => serde::private::de::missing_field("permissions")?,
                };
                let hash = match hash {
                    serde::export::Some(Hash) => Hash,
                    serde::export::None => serde::private::de::missing_field("hash")?,
                };
                serde::export::Ok(Credentials{
                    level: PhantomData,
                    permissions: permissions,
                    hash: hash,
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["permissions", "hash"];
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Credentials",
            FIELDS,
            Visitor {
                lifetime: serde::export::PhantomData,
            },
        )
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
/// creates an account, or logs a user into an account
#[post("/login")]
pub fn create(
    db: db::DB,
    login_data: Basic,
    cookies: Cookies,
    credentials: Option<Credentials<AnyPermissions>>,
    desired_permissions: Option<PermissionVec>,
) -> status::Custom<()> {
    db.create_user(basic);
    if credentials.is_some() {
        create_new(db, password, user);
    } else {
    }
    Status::Custom(Status::new(501, "Not yet implemented"), ())
}
/// deletes an account
#[delete("/login")]
pub fn delete(db: db::DB) -> status::Custom<()> {
    db.delete_user(db::find_user_by_hash()?);
    Status::Custom(Status::new(501, "Not yet implemented"), ())
}

#[cfg(test)]
mod unit_tests {
}

