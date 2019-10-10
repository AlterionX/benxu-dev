//! Data structures holding pertinent login information per request.

use crate::{
    blog::{
        auth::{self, perms::Verifiable},
        db::{PWQuery, UserQuery},
        DB,
    },
    PWAlgo,
};
use blog_db::models::*;
use boolinator::Boolinator;
use crypto::algo::{hash::symmetric::Algo as HashA, Algo as A};
pub use login_enum::CreatePassword;

/// Used to mark structs that can be converted into a database record and saved or used to update a
/// preexisting row in the table.
pub trait SavableCredential {
    /// The object returned on succcess, typically the ORM's `Data` representation of the struct.
    type Success;
    /// The object returned on succcess, typically the ORM's `Error` type.
    type Error;
    /// Converts the credential and attempts to create a new row for the credential. Will return
    /// the created row on success.
    fn convert_and_save_with_credentials(self) -> Result<Self::Success, Self::Error>;
    /// Converts the credential and attempts to update an existing row for the credential. Will
    /// return the updated row on success.
    fn convert_and_update_with_credentials(self) -> Result<Self::Success, Self::Error>;
}

/// A view into [`Password`](crate::blog::credentials::data::Password) together with the database
/// used to store credentials, and the secret key for the password hash.
pub struct PasswordWithBackingInfo<'a> {
    /// A reference to the [`DB`](crate::blog::DB) we will be using for verification.
    pub(super) db: &'a DB,
    /// A reference to the [`Credentials`](crate::blog::auth::Credentials) related to the request.
    pub(super) credentials: &'a auth::UnverifiedPermissionsCredential,
    /// A reference to the secret key for the password hashing.
    pub(super) argon2d_key: &'a <PWAlgo as A>::Key,
    /// A reference to the password credential data. Notice that this is not just a [`String`].
    pub(super) pw: &'a CreatePassword,
}
impl<'a> PasswordWithBackingInfo<'a> {
    /// Checks to ensure that the credentials provided matches the (assumed) owner of the password
    /// to be changed. This means that the credentials have the
    /// [`CanEditUserCredentials`](crate::blog::auth::perms::CanEditUserCredentials) permissions or
    /// that the credentials belong to the (assumed) owner of the password.
    fn verify_requester(&self) -> bool {
        use log::*;
        debug!(
            "Attempting to match {:?} with {:?}.",
            self.credentials.user_id(),
            self.pw.user_id
        );
        debug!(
            "Simple check validates to {:?}",
            self.credentials.user_id() == self.pw.user_id
        );
        self.credentials.user_id() == self.pw.user_id
            || auth::perms::CanEditUserCredentials::verify(self.credentials)
    }
    /// Checks if there are duplicate password entries, aka multiple passwords per user. This
    /// should not be allowed, and this helps detecting such situations.
    fn verify_duplicates(&self, target_count: usize) -> Result<bool, diesel::result::Error> {
        use log::*;
        debug!(
            "Attempting to check for duplicate password entries for {:?}.",
            self.pw.user_id
        );
        let u = self.db.find_user_by_id(self.pw.user_id)?;
        debug!("Account located. Searching for passwords.");
        let instances = self.db.count_pw_by_user(&u)?;
        debug!(
            "Passwords counted. Found {} items. Should find {} items.",
            instances, target_count
        );
        Ok(instances as usize == target_count)
    }
    /// Verifies the requester and the duplicate count as mentioned in
    /// [`verify_requester`](crate::blog::credentials::data::PasswordWithBackingInfo::verify_requester)
    /// and
    /// [`verify_duplicates`](crate::blog::credentials::data::PasswordWithBackingInfo::verify_duplicates).
    fn verify(&self, duplicate_count: usize) -> Result<bool, diesel::result::Error> {
        Ok(self.verify_requester() && self.verify_duplicates(duplicate_count)?)
    }
    /// Hashes the password with a generated salt. Returns first the generated salt, then the
    /// hashed password.
    fn hash(&self) -> (Vec<u8>, Vec<u8>) {
        let msg = &<PWAlgo as HashA>::VerificationInput::new_default_hash_len(
            self.pw.password.as_bytes().to_vec(),
            None,
        );
        let generated_salt = msg.salt();
        let pw_hash = PWAlgo::new(None).sign(msg, self.argon2d_key);
        (generated_salt.to_vec(), pw_hash)
    }
}
impl<'a> SavableCredential for PasswordWithBackingInfo<'a> {
    type Success = ();
    type Error = ();
    fn convert_and_save_with_credentials(self) -> Result<Self::Success, Self::Error> {
        use log::*;
        debug!(
            "Saving credentials. Ensuring {:?} can edit and no duplicate password entries for {:?}.",
            self.credentials.user_id(),
            self.pw.user_id,
        );
        self.verify(0)
            .map_err(|_| ())
            .and_then(|b| b.as_result((), ()))?;
        debug!("Verified. Hashing.");
        let (generated_salt, pw_hash) = self.hash();
        debug!("Hashed. Saving.");
        let creation = self.db.create_pw_hash(credentials::pw::New {
            created_by: self.credentials.user_id(),
            updated_by: self.credentials.user_id(),
            user_id: self.pw.user_id,
            hash: base64::encode(pw_hash.as_slice()).as_str(),
            salt: base64::encode(generated_salt.as_slice()).as_str(),
        });
        debug!("Attempt: {:?}", creation);
        creation.map(|_| ()).map_err(|_| ())
    }
    fn convert_and_update_with_credentials(self) -> Result<Self::Success, Self::Error> {
        self.verify(1)
            .map_err(|_| ())
            .and_then(|b| b.as_result((), ()))?;
        let (generated_salt, pw_hash) = self.hash();
        self.db
            .update_pw_hash_for_user_id(
                self.pw.user_id,
                credentials::pw::Changed {
                    updated_by: self.credentials.user_id(),
                    hash: Some(base64::encode(pw_hash.as_slice())),
                    salt: Some(base64::encode(generated_salt.as_slice())),
                },
            )
            .map(|_| ())
            .map_err(|_| ())
    }
}
