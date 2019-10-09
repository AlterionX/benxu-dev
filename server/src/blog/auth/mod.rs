//! Structs and utility functions for handling authorization tokens.

pub mod permissions;
pub use permissions as perms;
pub use perms::Permission;
mod error;
pub use error::Error;

use rocket::{
    http::{Cookie, Cookies, Status},
    outcome::IntoOutcome,
    request::{FromRequest, Outcome, Request},
    State,
};
use serde::{Deserialize, Deserializer, Serialize};
use tap::*;
use std::{marker::PhantomData, ops::Deref, str};

use crate::{TokenKeyFixture, TokenKeyStore};
use crypto::{algo::Algo as A, token::paseto, Generational};

/// The name of the cookie holding the credentials to be deserialized.
pub const AUTH_COOKIE_NAME: &'static str = "_atk";

/// A struct representing the list of permissions a user has.
///
/// TODO query the database for the list of permissions instead of serializing it.
#[derive(Debug, Serialize)]
pub struct Credentials<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    permissions: Vec<Permission>,
    user_id: uuid::Uuid,
}
impl<L> Credentials<L> {
    /// Check if a list of permissions is satisfied.
    pub fn has_permissions(&self, req_perms: &[Permission]) -> bool {
        req_perms
            .iter()
            .all(|req_perm| self.permissions.contains(req_perm))
    }
    /// Gets the a copy of the id of the user this credential belongs to.
    pub fn user_id(&self) -> uuid::Uuid {
        self.user_id
    }
    /// Gets the a copy of the permissions of the user.
    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_slice()
    }
    /// Attempts to change the credential's level, returning the old credential on error
    /// (insufficient permissions) and the new credential on success.
    pub fn change_level<NewLevel: perms::Verifiable>(
        self,
    ) -> Result<Credentials<NewLevel>, Credentials<L>> {
        let Credentials {
            user_id,
            permissions,
            level,
        } = self;
        Credentials::new(user_id, permissions).map_err(|(user_id, permissions)| Self {
            level: level,
            user_id: user_id,
            permissions: permissions,
        })
    }
    /// Revert the credential back to an unverified state.
    pub fn back_to_any(self) -> Credentials<perms::Any> {
        Credentials::safe_new(self.user_id, self.permissions)
    }
}
impl<L: perms::Verifiable> Credentials<L> {
    /// Creates a new Credentials object from a set of permissions and validates the permissions
    /// at the level requested by `L`.
    pub fn new(
        user_id: uuid::Uuid,
        permissions: Vec<Permission>,
    ) -> Result<Self, (uuid::Uuid, Vec<Permission>)> {
        if L::verify_slice(permissions.as_slice()) {
            Ok(Self {
                level: PhantomData,
                user_id: user_id,
                permissions: permissions,
            })
        } else {
            Err((user_id, permissions))
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
    /// Extracts an unverified credential from a provided token.
    fn extract(
        cookies: &Cookies,
        key_store: &TokenKeyStore,
    ) -> Result<Credentials<perms::Any>, Error> {
        let auth_cookie = cookies.get(AUTH_COOKIE_NAME).ok_or(Error::Unauthorized)?;

        type TokenData = paseto::token::Data<Credentials<perms::Any>, ()>;
        // TODO no-copy once paseto is no copy on the input
        let token: TokenData = key_store.attempt_with_retry(&mut |key, _opt_err| {
            let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
            paseto::v2::local::Protocol::decrypt(token, key)
        })?;

        Ok(token.msg)
    }
}
/// Custom implementation of Deserialize is due the need to forbid deserialization of credentials
/// into arbitrary permission levels.
impl<'de> Deserialize<'de> for Credentials<perms::Any> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Permissions,
            UserId,
            Ignore,
        }
        struct FieldVisitor;
        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(
                &self,
                formatter: &mut serde::export::Formatter,
            ) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "field identifier")
            }
            fn visit_u64<E>(self, value: u64) -> serde::export::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0u64 => serde::export::Ok(Field::Permissions),
                    1u64 => serde::export::Ok(Field::UserId),
                    _ => serde::export::Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Unsigned(value),
                        &"field index 0 <= i < 2",
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> serde::export::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "permissions" => serde::export::Ok(Field::Permissions),
                    "user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> serde::export::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"permissions" => serde::export::Ok(Field::Permissions),
                    b"user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore),
                }
            }
        }
        impl<'de> serde::Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> serde::export::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct Visitor<'de> {
            lifetime: serde::export::PhantomData<&'de ()>,
        }
        impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Credentials<perms::Any>;
            fn expecting(
                &self,
                formatter: &mut serde::export::Formatter,
            ) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "struct Credentials")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let permissions = serde::de::SeqAccess::next_element::<Vec<Permission>>(&mut seq)?
                    .ok_or(serde::de::Error::invalid_length(
                        0usize,
                        &"struct Credentials with 2 elements",
                    ))?;
                let user_id = serde::de::SeqAccess::next_element::<uuid::Uuid>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(1usize, &"struct Credentials with 2 elements"),
                )?;
                Ok(Credentials {
                    level: PhantomData,
                    permissions: permissions,
                    user_id: user_id,
                })
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> serde::export::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut permissions = None;
                let mut user_id = None;
                while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Permissions => {
                            permissions = if permissions.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "permissions",
                                ));
                            } else {
                                Some(serde::de::MapAccess::next_value::<Vec<Permission>>(
                                    &mut map,
                                )?)
                            }
                        }
                        Field::UserId => {
                            user_id = if user_id.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "user_id",
                                ));
                            } else {
                                Some(serde::de::MapAccess::next_value::<uuid::Uuid>(&mut map)?)
                            }
                        }
                        _ => {
                            let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(
                                &mut map,
                            )?;
                        }
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
                serde::export::Ok(Credentials {
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
impl<'a, 'r, L: perms::Verifiable + std::fmt::Debug> FromRequest<'a, 'r> for Credentials<L> {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        req.guard::<UnverifiedPermissionsCredential>()
            .tap(|res| log::debug!("Found credentials? {:?}", res))?
            .into_inner()
            .change_level()
            .tap(|res| log::debug!("Level changed: {:?}", res))
            .map_err(|_| Error::Unauthorized)
            .into_outcome(Status::Unauthorized)
    }
}

/// A wrapper around [`Credential<()>`](crate::blog::auth::Credentials) for ensuring awareness of
/// the lack of permissions.
#[derive(Debug, Deserialize)]
pub struct UnverifiedPermissionsCredential(Credentials<perms::Any>);
impl UnverifiedPermissionsCredential {
    /// Create a new struct, simply wrapping up the
    /// [`Credentials::new()`](crate::blog::auth::Credentials::new) function with the newtype
    /// struct.
    pub fn new(user_id: uuid::Uuid, perms: Vec<Permission>) -> Self {
        // Should not be able to error
        Self(Credentials::safe_new(user_id, perms))
    }
    /// Consumes self, yielding a [`Credentials<Any>`](crate::blog::auth::Credentials) struct.
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
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let key_store = req
            .guard::<State<TokenKeyFixture>>()
            .map_failure(|_| Error::KeyStoreAbsent.into())?
            .get_store()
            .map_err(|_| Error::KeyStoreAbsent.into())
            .into_outcome(Status::InternalServerError)?;

        Credentials::extract(&cookies, &*key_store)
            .into_outcome(Status::Unauthorized)
            .map(|cr: Credentials<perms::Any>| cr.into())
    }
}
impl From<Credentials<perms::Any>> for UnverifiedPermissionsCredential {
    fn from(cr: Credentials<perms::Any>) -> Self {
        log::debug!("{:?}", cr);
        Self(cr)
    }
}

/// Attaches a [`Credentials`](crate::blog::auth::Credentials) to the cookies so that they
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
    let token_str = paseto::v2::local::Protocol
        .encrypt(tok, key)
        .map_err(|_| ())
        .and_then(|s| Ok(str::from_utf8(&s).map_err(|_| ())?.to_owned()))?;
    let auth_cookie = Cookie::build(AUTH_COOKIE_NAME, token_str)
        .secure(true)
        .http_only(true)
        .finish();
    cookies.add(auth_cookie);
    Ok(())
}
/// Detaches a [`Credentials`](crate::blog::auth::Credentials) from the cookie.
pub fn detach_credentials_token_if_exists(cookies: &mut Cookies) {
    let auth_cookie = cookies.get(AUTH_COOKIE_NAME);
    if auth_cookie.is_some() {
        cookies.remove(Cookie::named(AUTH_COOKIE_NAME));
    }
}
