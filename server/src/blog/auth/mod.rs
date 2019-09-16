//! Structs and utility functions for handling authorization tokens.

pub mod permissions;
pub use permissions as perms;
pub use perms::Permission as Permission;
mod error;
pub use error::Error;

use std::{
    marker::PhantomData,
    ops::Deref,
    str,
};
use rocket::{
    http::{Status, Cookies, Cookie},
    request::{
        Request,
        FromRequest,
        Outcome,
    },
    outcome::IntoOutcome,
    State,
};
use serde::{
    Serialize,
    Deserializer,
    Deserialize,
};

use crypto::{
    Generational,
    algo::Algo as A,
    token::paseto,
};
use crate::{
    TokenKeyFixture,
    TokenKeyStore,
};

/// The name of the cookie holding the credentials to be deserialized.
pub const AUTH_COOKIE_NAME: &'static str = "_atk";

/// A struct representing the list of permissions a user has.
///
/// TODO query the database for the list of permissions instead of serializing it.
#[derive(Serialize)]
pub struct Credentials<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    permissions: Vec<Permission>,
    user_id: uuid::Uuid,
}
impl<L> Credentials<L> {
    /// Check if a list of permissions is satisfied.
    pub fn has_permissions(&self, req_perms: &[Permission]) -> bool {
        for req_perm in req_perms.iter() {
            if !self.permissions.contains(req_perm) {
                return false;
            }
        }
        true
    }
    /// Gets the a copy of the id of the user this credential belongs to.
    pub fn user_id(&self) -> uuid::Uuid {
        self.user_id
    }
    /// Gets a reference to the permissions in the credential.
    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_slice()
    }
    /// Attempts to change the credential's level, returning the old credential on error
    /// (insufficient permissions) and the new credential on success.
    pub fn change_level<NewLevel: perms::Verifiable>(self) -> Result<Credentials<NewLevel>, Credentials<L>> {
        // TODO make permissions list no-copy
        Credentials::<NewLevel>::new(self.user_id, self.permissions.clone()).ok_or(self)
    }
    /// Revert the credential back to an unverified state.
    pub fn back_to_any(self) -> Credentials<perms::Any> {
        Credentials::safe_new(self.user_id, self.permissions)
    }
}
impl<L: perms::Verifiable> Credentials<L> {
    /// Extracts an unverified credential from a provided token.
    fn extract(cookies: &Cookies, key: &TokenKeyStore) -> Result<Credentials<perms::Any>, Error> {
        let auth_cookie = cookies.get(AUTH_COOKIE_NAME).ok_or(Error::Unauthorized)?;
        let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());

        let token: paseto::token::Data<Credentials<perms::Any>, ()> = match paseto::v2::local::Protocol::decrypt(token, &key.curr) {
            Ok(dec) => dec,
            Err(paseto::v2::local::error::Error::Decryption) => {
                let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
                paseto::v2::local::Protocol::decrypt(token, &key.last)
                    .map_err(|_| Error::Unauthorized)?
            },
            _ => return Err(Error::Unauthorized),
        };
        Ok(token.msg)
    }
    /// Creates a new Credentials object from a set of permissions and validates the permissions
    /// at the level requested by `L`.
    pub fn new(user_id: uuid::Uuid, permissions: Vec<Permission>) -> Option<Self> {
        if L::verify_slice(permissions.as_slice()) {
            Some(Self {
                level: PhantomData,
                user_id: user_id,
                permissions: permissions,
            })
        } else {
            None
        }
    }
}
impl Credentials<perms::Any> {
    /// Creating an unverified credential has no chance of failure; this allows for the provided
    /// specialization of no error.
    pub fn safe_new(user_id: uuid::Uuid, permissions: Vec<Permission>) -> Self {
        Self {
            level: PhantomData,
            user_id: user_id,
            permissions: permissions,
        }
    }
}
/// Custom implementation of Deserialize is due the need to forbid deserialization of credentials
/// into arbitrary permission levels.
impl <'de> Deserialize<'de> for Credentials<perms::Any> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        enum Field {
            Permissions,
            UserId,
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
                    1u64 => serde::export::Ok(Field::UserId),
                    _ => serde::export::Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(value),
                            &"field index 0 <= i < 2"
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    "permissions" => serde::export::Ok(Field::Permissions),
                    "user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore)
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    b"permissions" => serde::export::Ok(Field::Permissions),
                    b"user_id" => serde::export::Ok(Field::UserId),
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
            type Value = Credentials<perms::Any>;
            fn expecting(&self, formatter: &mut serde::export::Formatter) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "struct Credentials")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de> {
                let permissions = serde::de::SeqAccess::next_element::<Vec<Permission>>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(0usize, &"struct Credentials with 2 elements")
                )?;
                let user_id = serde::de::SeqAccess::next_element::<uuid::Uuid>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(1usize, &"struct Credentials with 2 elements")
                )?;
                Ok(
                    Credentials{
                        level: PhantomData,
                        permissions: permissions,
                        user_id: user_id,
                    }
                )
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> serde::export::Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
                let mut permissions = None;
                let mut user_id = None;
                while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Permissions => permissions = if permissions.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("permissions"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<Vec<Permission>>(&mut map)?)
                        },
                        Field::UserId => user_id = if user_id.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("user_id"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<uuid::Uuid>(&mut map)?)
                        },
                        _ => { let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(&mut map)?; },
                    }
                }
                let permissions = match permissions {
                    serde::export::Some(permissions) => permissions,
                    serde::export::None => serde::private::de::missing_field("permissions")?,
                };
                let user_id = match user_id {
                    serde::export::Some(user_id) => user_id,
                    serde::export::None => serde::private::de::missing_field("user_id")?,
                };
                serde::export::Ok(Credentials{
                    level: PhantomData,
                    permissions: permissions,
                    user_id: user_id,
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["permissions", "user_id"];
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
impl<'a, 'r, L: perms::Verifiable> FromRequest<'a, 'r> for Credentials<L> {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let key_store = req
            .guard::<State<TokenKeyFixture>>()
            .map_failure(|e| Error::KeyStoreAbsent.into())?
            .get_store()
            .map_err(|_| Error::KeyStoreAbsent.into())
            .into_outcome(Status::InternalServerError)?;

        let cr = Self::extract(&cookies, &*key_store)
            .map_err(|e| e.into())
            .into_outcome(Status::Unauthorized)?;

        if L::verify(&cr) {
            Outcome::Success(Credentials {
                level: PhantomData,
                permissions: cr.permissions,
                user_id: cr.user_id,
            })
        } else {
            Outcome::Failure(Error::Unauthorized.into())
        }
    }
}

/// A wrapper around [`Credential<()>`](crate::blog::auth::Credentials) for ensuring awareness of
/// the lack of permissions.
#[derive(Deserialize)]
pub struct UnverifiedPermissionsCredential(Credentials<perms::Any>);
impl UnverifiedPermissionsCredential {
    pub fn new(user_id: uuid::Uuid, perms: Vec<Permission>) -> Self {
        // Should not be able to error
        Self(Credentials::new(user_id, perms).unwrap())
    }
    pub fn into_inner(self) -> Credentials<perms::Any> {
        self.0
    }
}
impl Deref for UnverifiedPermissionsCredential {
    type Target = Credentials<perms::Any>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for UnverifiedPermissionsCredential {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cr = req
            .guard::<Credentials<()>>()
            .map_failure(|_| (Status::InternalServerError, ()))?;
        Outcome::Success(Self(cr))
    }
}
impl From<Credentials<perms::Any>> for UnverifiedPermissionsCredential {
    fn from(cr: Credentials<perms::Any>) -> Self {
        Self(cr)
    }
}

/// Attaches a [`CredentialToken`](crate::blog::auth::CredentialToken) to the cookies so that they
/// can be verified later.
#[must_use]
pub fn attach_credentials_token(
    key: &<paseto::v2::local::Algo as A>::Key,
    credentials: Credentials<perms::Any>,
    cookies: &mut Cookies,
) -> Result<(), ()> {
    detach_credentials_token_if_exists(cookies);
    let tok = paseto::token::Data {
        msg: credentials,
        footer: (None: Option<()>),
    };
    let token_str = paseto::v2::local::Protocol.encrypt(tok, key)
        .map_err(|_| ())
        .and_then(|s| Ok(str::from_utf8(&s).map_err(|_| ())?.to_owned()))?;
    let auth_cookie = Cookie::build(AUTH_COOKIE_NAME, token_str)
        .secure(true)
        .http_only(true)
        .finish();
    cookies.add(auth_cookie);
    Ok(())
}
/// Detaches a [`CredentialToken`](crate::blog::auth::CredentialToken) from the cookie.
pub fn detach_credentials_token_if_exists(cookies: &mut Cookies) {
    let auth_cookie = cookies.get(AUTH_COOKIE_NAME);
    if auth_cookie.is_some() {
        cookies.remove(Cookie::named(AUTH_COOKIE_NAME));
    }
}

