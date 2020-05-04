//! Structs and utility functions for handling authorization tokens.

pub mod capabilities;
pub use capabilities as caps;
pub use caps::Capability;
mod error;
pub use error::Error;
pub mod credentials;

use rocket::{
    http::{Cookie, Cookies, Status},
    outcome::IntoOutcome,
    request::{FromRequest, Outcome, Request},
    State,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{marker::PhantomData, ops::Deref, str};
use tap::*;

use crate::cfg::{TokenKeyFixture, TokenKeyStore};
use crypto::{
    algo::Algo as A,
    key_rotation::Generational,
    token::paseto::{self, Protocol},
};

/// The name of the cookie holding the capabilities to be deserialized.
pub const AUTH_COOKIE_NAME: &str = "_atk";

/// A struct representing the list of capabilities a user has.
///
/// TODO query the database for the list of capabilities instead of serializing it.
#[derive(Debug, Serialize)]
pub struct Capabilities<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    capabilities: Vec<Capability>,
    user_id: uuid::Uuid,
}
impl<L> Capabilities<L> {
    /// Check if a list of capabilities is satisfied.
    pub fn has_capabilities(&self, req_perms: &[Capability]) -> bool {
        req_perms
            .iter()
            .all(|req_perm| self.capabilities.contains(req_perm))
    }
    /// Gets the a copy of the id of the user this credential belongs to.
    pub fn user_id(&self) -> uuid::Uuid {
        self.user_id
    }
    /// Gets the a copy of the capabilities of the user.
    pub fn capabilities(&self) -> &[Capability] {
        self.capabilities.as_slice()
    }
    /// Attempts to change the credential's level, returning the old credential on error
    /// (insufficient capabilities) and the new credential on success.
    pub fn change_level<NewLevel: caps::Verifiable>(
        self,
    ) -> Result<Capabilities<NewLevel>, Capabilities<L>> {
        let Capabilities {
            user_id,
            capabilities,
            level,
        } = self;
        Capabilities::new(user_id, capabilities).map_err(|(user_id, capabilities)| Self {
            level,
            user_id,
            capabilities,
        })
    }
    /// Revert the credential back to an unverified state.
    pub fn back_to_any(self) -> Capabilities<caps::Any> {
        Capabilities::safe_new(self.user_id, self.capabilities)
    }
}
impl<L: caps::Verifiable> Capabilities<L> {
    /// Creates a new Capabilities object from a set of capabilities and validates the capabilities
    /// at the level requested by `L`.
    pub fn new(
        user_id: uuid::Uuid,
        capabilities: Vec<Capability>,
    ) -> Result<Self, (uuid::Uuid, Vec<Capability>)> {
        if L::verify_slice(capabilities.as_slice()) {
            Ok(Self {
                level: PhantomData,
                user_id,
                capabilities,
            })
        } else {
            Err((user_id, capabilities))
        }
    }
}
impl Capabilities<caps::Any> {
    /// Creating an unverified credential has no chance of failure; this allows for the provided
    /// specialization of no error.
    pub fn safe_new(user_id: uuid::Uuid, capabilities: Vec<Capability>) -> Self {
        Self {
            level: PhantomData,
            user_id,
            capabilities,
        }
    }
    /// Extracts an unverified credential from a provided token.
    fn extract(
        cookies: &Cookies,
        key_store: &TokenKeyStore,
    ) -> Result<Capabilities<caps::Any>, Error> {
        let auth_cookie = cookies.get(AUTH_COOKIE_NAME).ok_or(Error::Unauthorized)?;

        type TokenData = paseto::token::Data<Capabilities<caps::Any>, ()>;
        // TODO no-copy once paseto is no copy on the input
        let token: TokenData = key_store.attempt_with_retry(&mut |key, _opt_err| {
            let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
            paseto::V2Local::decrypt(token, key)
        })?;

        Ok(token.msg)
    }
}
/// Custom implementation of Deserialize is due the need to forbid deserialization of capabilities
/// into arbitrary capability levels.
impl<'de> Deserialize<'de> for Capabilities<caps::Any> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Capabilities,
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
                    0u64 => serde::export::Ok(Field::Capabilities),
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
                    "capabilities" => serde::export::Ok(Field::Capabilities),
                    "user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> serde::export::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"capabilities" => serde::export::Ok(Field::Capabilities),
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
            type Value = Capabilities<caps::Any>;
            fn expecting(
                &self,
                formatter: &mut serde::export::Formatter,
            ) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "struct Capabilities")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let capabilities = serde::de::SeqAccess::next_element::<Vec<Capability>>(&mut seq)?
                    .ok_or_else(|| {
                        serde::de::Error::invalid_length(
                            0usize,
                            &"struct Capabilities with 2 elements",
                        )
                    })?;
                let user_id = serde::de::SeqAccess::next_element::<uuid::Uuid>(&mut seq)?
                    .ok_or_else(|| {
                        serde::de::Error::invalid_length(
                            1usize,
                            &"struct Capabilities with 2 elements",
                        )
                    })?;
                Ok(Capabilities {
                    level: PhantomData,
                    capabilities,
                    user_id,
                })
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> serde::export::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut capabilities = None;
                let mut user_id = None;
                while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Capabilities => {
                            capabilities = if capabilities.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "capabilities",
                                ));
                            } else {
                                Some(serde::de::MapAccess::next_value::<Vec<Capability>>(
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
                let capabilities = match capabilities {
                    serde::export::Some(capabilities) => capabilities,
                    serde::export::None => serde::private::de::missing_field("capabilities")?,
                };
                let user_id = match user_id {
                    serde::export::Some(user_id) => user_id,
                    serde::export::None => serde::private::de::missing_field("user_id")?,
                };
                serde::export::Ok(Capabilities {
                    level: PhantomData,
                    capabilities,
                    user_id,
                })
            }
        }
        const FIELDS: &[&str] = &["capabilities", "user_id"];
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Capabilities",
            FIELDS,
            Visitor {
                lifetime: serde::export::PhantomData,
            },
        )
    }
}
impl<'a, 'r, L: caps::Verifiable + std::fmt::Debug> FromRequest<'a, 'r> for Capabilities<L> {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        req.guard::<UnverifiedCapabilities>()
            .tap(|res| log::debug!("Found capabilities? {:?}", res))?
            .into_inner()
            .change_level()
            .tap(|res| log::debug!("Level changed: {:?}", res))
            .map_err(|_| Error::Unauthorized)
            .into_outcome(Status::Unauthorized)
    }
}

impl<L> Clone for Capabilities<L> {
    fn clone(&self) -> Self {
        Capabilities {
            level: PhantomData,
            capabilities: self.capabilities.clone(),
            user_id: self.user_id.clone(),
        }
    }
}

/// A wrapper around [`Credential<()>`](crate::blog::auth::Capabilities) for ensuring awareness of
/// the lack of capabilities.
#[derive(Debug, Deserialize)]
pub struct UnverifiedCapabilities(Capabilities<caps::Any>);
impl UnverifiedCapabilities {
    /// Create a new struct, simply wrapping up the
    /// [`Capabilities::new()`](crate::blog::auth::Capabilities::new) function with the newtype
    /// struct.
    pub fn new(user_id: uuid::Uuid, caps: Vec<Capability>) -> Self {
        // Should not be able to error
        Self(Capabilities::safe_new(user_id, caps))
    }
    /// Consumes self, yielding a [`Capabilities<Any>`](crate::blog::auth::Capabilities) struct.
    pub fn into_inner(self) -> Capabilities<caps::Any> {
        self.0
    }
}
impl Deref for UnverifiedCapabilities {
    type Target = Capabilities<caps::Any>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for UnverifiedCapabilities {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let key_store = req
            .guard::<State<TokenKeyFixture>>()
            .map_failure(|_| Error::KeyStoreAbsent.into())?
            .get_store()
            .map_err(|_| Error::KeyStoreAbsent)
            .into_outcome(Status::InternalServerError)?;

        Capabilities::extract(&cookies, &*key_store)
            .into_outcome(Status::Unauthorized)
            .map(|cr: Capabilities<caps::Any>| cr.into())
    }
}
impl From<Capabilities<caps::Any>> for UnverifiedCapabilities {
    fn from(cr: Capabilities<caps::Any>) -> Self {
        log::debug!("{:?}", cr);
        Self(cr)
    }
}

/// Attaches a [`Capabilities`](crate::blog::auth::Capabilities) to the cookies so that they
/// can be verified later.
#[must_use]
pub fn attach_capabilities_token(
    key: &<<paseto::V2Local as paseto::Protocol>::CoreAlgo as A>::Key,
    capabilities: Capabilities<caps::Any>,
    cookies: &mut Cookies,
) -> Result<(), ()> {
    detach_capabilities_token_if_exists(cookies);
    let opt_none: Option<()> = None;
    let tok = paseto::token::Data {
        msg: capabilities,
        footer: opt_none,
    };
    let token_str = paseto::V2Local::encrypt(tok, key)
        .map_err(|_| ())
        .and_then(|s| Ok(str::from_utf8(&s).map_err(|_| ())?.to_owned()))?;
    let auth_cookie = Cookie::build(AUTH_COOKIE_NAME, token_str)
        .secure(true)
        .http_only(true)
        .finish();
    cookies.add(auth_cookie);
    Ok(())
}
/// Detaches a [`Capabilities`](crate::blog::auth::Capabilities) from the cookie.
pub fn detach_capabilities_token_if_exists(cookies: &mut Cookies) {
    let auth_cookie = cookies.get(AUTH_COOKIE_NAME);
    if auth_cookie.is_some() {
        cookies.remove(Cookie::named(AUTH_COOKIE_NAME));
    }
}
