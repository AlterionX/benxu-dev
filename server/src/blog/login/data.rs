//! Data structures and functions representing login credentials.

use boolinator::Boolinator;

use crate::{
    blog::{
        auth,
        db::{PWQuery, PermissionQuery, UserQuery},
        DB,
    },
    PWAlgo, PWKeyFixture,
};
use blog_db::models::*;
use crypto::algo::{hash::symmetric::Algo as HashA, Algo as A};
pub use login_enum::*;

/// Encodes a pairing of input and stored credentials of same type.
pub enum AuthnWithStored<'a> {
    Password(&'a Password, credentials::pw::Data),
}
impl<'a> AuthnWithStored<'a> {
    /// Verify a credential against the stored version. This is currently specific to passwords.
    fn verify_with_err(self, key: &<PWAlgo as A>::Key) -> Result<(), ()> {
        use log::*;
        match self {
            Self::Password(pw, hash_and_salt) => {
                debug!("Decode pw from base64.");
                let hash = base64::decode(hash_and_salt.hash.as_bytes()).map_err(|_| ())?;
                let salt = base64::decode(hash_and_salt.salt.as_bytes()).map_err(|_| ())?;
                let salt = {
                    let mut buf = [0; PWAlgo::SALT_LEN as usize];
                    buf.copy_from_slice(salt.as_slice());
                    buf
                };
                let hash_input = <PWAlgo as HashA>::VerificationInput::new(
                    pw.password.as_bytes().to_vec(),
                    Some(salt),
                    Some(hash.len() as u32),
                )
                .map_err(|_| ())?;
                trace!("attempting verification.");
                PWAlgo::new(None)
                    .verify(&hash_input, hash.as_slice(), key)
                    .as_result((), ())
            }
        }
    }
}

pub trait CanAuthenticate {
    #[must_use]
    /// Authenticates users and gets permissions of user if successful.
    fn authenticate(
        &self,
        db: &DB,
        pw_key_store: &PWKeyFixture,
    ) -> Result<(users::Data, Vec<auth::Permission>), auth::Error>;
    /// Find user this credential belongs to along with a list of permissions belonging to the
    /// user.
    fn find_targeted_user(
        &self,
        db: &DB,
    ) -> Result<(users::Data, Vec<permissions::Data>), diesel::result::Error>;
    /// Create a reference of the submitted credentials alongside the official credentials. This
    /// will be verified later on.
    fn pair_with_stored(
        &self,
        db: &DB,
        user: &users::Data,
    ) -> Result<AuthnWithStored, diesel::result::Error>;
}
impl CanAuthenticate for Authentication {
    fn authenticate(
        &self,
        db: &DB,
        pw_key_store: &PWKeyFixture,
    ) -> Result<(users::Data, Vec<auth::Permission>), auth::Error> {
        use log::*;
        trace!("Beginning authentication process.");
        let (user, perms) = self.find_targeted_user(db)?;
        trace!("Found user.");
        let key = pw_key_store.key();
        trace!("Found secret key.");
        let targeted_credential = self.pair_with_stored(db, &user)?;
        trace!("Found secret key.");
        targeted_credential
            .verify_with_err(&*key)
            .map(|_| (user, perms.iter().map(auth::Permission::from).collect()))
            .map_err(|_| auth::Error::BadCredentials)
    }
    fn find_targeted_user(
        &self,
        db: &DB,
    ) -> Result<(users::Data, Vec<permissions::Data>), diesel::result::Error> {
        use log::*;
        trace!("Beginning user search.");
        let user = match self {
            Self::Password(p) => db.find_user_by_user_name(p.user_name.as_str()),
        }?;
        trace!("Getting permissions for user.");
        let permissions = db.get_user_permissions(&user)?;
        trace!("Both located. Returning.");
        Ok((user, permissions))
    }
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
