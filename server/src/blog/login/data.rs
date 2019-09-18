//! Data structures and functions representing login credentials.

use std::str;

use boolinator::Boolinator;
use serde::{Deserialize, Serialize};

use crate::{
    blog::{auth, DB},
    PWAlgo, PWKeyFixture,
};
use blog_db::models::*;
use crypto::algo::{hash::symmetric::Algo as HashA, Algo as A};

/// Encodes a pairing of input and stored credentials of same type.
pub enum AuthnWithStored<'a> {
    Password(&'a Password, credentials::pw::Data),
}
impl<'a> AuthnWithStored<'a> {
    /// Verify a credential against the stored version. This is currently specific to passwords.
    fn verify_with_err(self, key: &<PWAlgo as A>::Key) -> Result<(), ()> {
        match self {
            Self::Password(pw, hash_and_salt) => {
                let pw = base64::decode(pw.password.as_bytes()).map_err(|_| ())?;
                let salt = {
                    let mut buf = [0; PWAlgo::SALT_LEN as usize];
                    buf.copy_from_slice(hash_and_salt.salt.as_bytes());
                    buf
                };
                let hash_input = <PWAlgo as HashA>::VerificationInput::new(
                    pw,
                    Some(salt),
                    Some(hash_and_salt.hash.len() as u32),
                )
                .map_err(|_| ())?;
                PWAlgo::new(None).verify(&hash_input, hash_and_salt.hash.as_bytes(), key)
                    .as_result((), ())
            }
        }
    }
}

/// Password authentication data. Separated from AuthenticationData to allow for impl blocks. Will
/// go away once enum variants become types.
#[derive(Serialize, Deserialize)]
pub struct Password {
    user_name: String,
    password: String,
}

/// Actual data that needs to be verified before someone can log in.
/// Currently only allows for passwords, but planning to support SSO and FIDO.
#[derive(Serialize, Deserialize)]
pub enum Authentication {
    /// Data needed to fully specify a password credential from the request.
    Password(Password),
}
impl Authentication {
    // TODO ship creation code to crate::blog::accounts.
    #[must_use]
    /// Authenticates users and gets permissions of user if successful.
    pub fn authenticate(
        &self,
        db: &DB,
        pw_key_store: &PWKeyFixture,
    ) -> Result<(users::Data, Vec<auth::Permission>), auth::Error> {
        let (user, perms) = self.find_targeted_user(db)?;
        let key = pw_key_store.key();
        let targeted_credential = self.pair_with_stored(db, &user)?;
        targeted_credential
            .verify_with_err(&*key)
            .map(|_| {
                (
                    user,
                    perms.iter().map(|p| auth::Permission::from(p)).collect(),
                )
            })
            .map_err(|_| auth::Error::BadCredentials)
    }
    /// Find user this credential belongs to along with a list of permissions belonging to the
    /// user.
    fn find_targeted_user(
        &self,
        db: &DB,
    ) -> Result<(users::Data, Vec<permissions::Data>), diesel::result::Error> {
        let user = match self {
            Self::Password(p) => db.find_user_by_user_name(p.user_name.as_str()),
        }?;
        let permissions = db.get_user_permissions(&user)?;
        Ok((user, permissions))
    }
    /// Create a reference of the submitted credentials alongside the official credentials. This
    /// will be verified later on.
    fn pair_with_stored(
        &self,
        db: &DB,
        user: &users::Data,
    ) -> Result<AuthnWithStored, diesel::result::Error> {
        match self {
            Self::Password(p) => db
                .find_pw_hash_by_user(user)
                .map(move |d| AuthnWithStored::Password(p, d)),
        }
    }
}
