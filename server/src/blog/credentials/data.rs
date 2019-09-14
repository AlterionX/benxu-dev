use serde::{
    Serialize,
    Deserialize,
};
use blog_db::models::*;
use crate::{
    crypto::{
        algo::{
            Algo as A,
            hash::{
                symmetric::Algo as HashA,
                argon2::d::{Algo as ARGON2D, SigningData as ARGON2D_MSG},
            },
        },
    },
    blog::{
        db,
        auth::{
            self,
            perms::Verifiable,
        },
    },
};

#[derive(Serialize, Deserialize)]
pub struct Password {
    user_id: uuid::Uuid,
    password: String,
}
pub struct PasswordWithBackingInfo<'a> {
    pub(super) db: &'a db::DB,
    pub(super) credentials: &'a auth::UnverifiedPermissionsCredential,
    pub(super) argon2d_key: &'a <ARGON2D as A>::Key,
    pub(super) pw: &'a Password,
}
pub trait SavableCredential {
    type Success;
    type Error;
    fn convert_and_save_with_credentials(self) -> Result<Self::Success, Self::Error>;
    fn convert_and_update_with_credentials(self) -> Result<Self::Success, Self::Error>;
}
impl<'a> PasswordWithBackingInfo<'a> {
    fn verify_requester(&self) -> bool {
        self.credentials.user_id() == self.pw.user_id || auth::perms::CanEditUserCredentials::verify(self.credentials)
    }
    fn verify_duplicates(&self, num: usize) -> Result<bool, diesel::result::Error> {
        let instances = self.db.count_pw_by_user(&self.db.find_user_by_id(self.pw.user_id)?)?;
        Ok(instances == num)
    }
    fn verify(&self, num: usize) -> Result<bool, diesel::result::Error> {
        Ok(self.verify_requester() && self.verify_duplicates(num)?)
    }
    fn hash(&self) -> (Vec<u8>, Vec<u8>) {
        let msg = &ARGON2D_MSG::new_default_hash_len(
            self.pw.password.as_bytes().to_vec(),
            None,
        );
        let generated_salt = msg.salt();
        let pw_hash = <ARGON2D as HashA>::sign(
            msg,
            self.argon2d_key,
        );
        (generated_salt.to_vec(), pw_hash)
    }
}
impl<'a> SavableCredential for PasswordWithBackingInfo<'a> {
    type Success = ();
    type Error = ();
    fn convert_and_save_with_credentials(self) -> Result<Self::Success, Self::Error> {
        if !self.verify(0).map_err(|_| ())? {
            return Err(());
        }
        let (generated_salt, pw_hash) = self.hash();
        self.db.create_pw_hash(credentials::pw::New {
            created_by: self.credentials.user_id(),
            updated_by: self.credentials.user_id(),
            user_id: self.pw.user_id,
            hash: base64::encode(pw_hash.as_slice()).as_str(),
            salt: base64::encode(generated_salt.as_slice()).as_str(),
        })
        .map(|_| ())
        .map_err(|_| ())
    }
    fn convert_and_update_with_credentials(self) -> Result<Self::Success, Self::Error> {
        if !self.verify(1).map_err(|_| ())? {
            return Err(());
        }
        let (generated_salt, pw_hash) = self.hash();
        self.db.update_pw_hash_for_user_id(self.pw.user_id, credentials::pw::Changed {
            updated_by: self.credentials.user_id(),
            hash: Some(base64::encode(pw_hash.as_slice())),
            salt: Some(base64::encode(generated_salt.as_slice())),
        })
        .map(|_| ())
        .map_err(|_| ())
    }
}

