use std::{
    io::Read,
    sync::Arc,
    str,
};

use serde::{
    Serialize,
    Deserialize,
};
use rocket::{
    response::{
        status,
        Redirect,
    },
    http::{Status, Cookie, Cookies, ContentType},
    request::{
        Request,
        Outcome,
    },
    data::{
        Outcome as OutcomeWithData,
        Data,
        FromDataSimple,
    },
    State,
};
use rocket_contrib::json::Json;

use blog_db::models::*;
use crate::{
    crypto::{
        KeyStore,
        algo::{
            Algo as A,
            hash::{
                symmetric::Algo as HashA,
                argon2::d::{Algo as ARGON2D, SigningData as ARGON2D_MSG},
            },
        },
        token::paseto,
    },
    blog::{
        db,
        accounts,
        auth::{
            self,
            perms::Verifiable,
        },
    },
};

/// Encodes a pairing of input and stored credentials of same type.
pub enum AuthnWithStored<'a> {
    Password(&'a Password, credentials::pw::Data),
}
impl<'a> AuthnWithStored<'a> {
    fn verify_with_err(self, key: &<ARGON2D as A>::Key) -> Result<(), ()> {
        match self {
            Self::Password(pw, hash_and_salt) => if <ARGON2D as HashA>::verify(
                &ARGON2D_MSG::new(
                    base64::decode(pw.password.as_bytes()).map_err(|_| ())?.clone(),
                    {
                        let mut buffer = [0; ARGON2D::SALT_LEN as usize];
                        buffer.copy_from_slice(hash_and_salt.salt.as_bytes());
                        Some(buffer)
                    },
                    Some(hash_and_salt.hash.len() as u32),
                ).map_err(|_| ())?,
                hash_and_salt.hash.as_bytes(),
                key,
            ) {
                Ok(())
            } else {
                Err(())
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
    /// Authenticates or creates user or modifies table.
    pub fn authenticate(
        &self,
        db: &db::DB,
        pw_key_store: &KeyStore<ARGON2D>,
    ) -> Result<(users::Data, Vec<auth::Permission>), auth::Error> {
        let (user, perms) = self.find_targeted_user(db)?;
        let key = &pw_key_store.curr_and_last().map_err(|_| auth::Error::KeyStorePoisoned)?.curr;
        let targeted_credential = self.pair_with_stored(db, &user)?;
        targeted_credential.verify_with_err(&key)
            .map(|_| (user, perms.iter().map(|p| auth::Permission::from(p)).collect()))
            .map_err(|_| auth::Error::BadCredentials)
    }
    fn find_targeted_user(&self, db: &db::DB) -> Result<(users::Data, Vec<permissions::Data>), diesel::result::Error> {
        let user = match self {
            Self::Password(p) => db.find_user_by_user_name(p.user_name.as_str()),
        }?;
        let permissions = db.get_user_permissions(&user)?;
        Ok((user, permissions))
    }
    fn pair_with_stored(&self, db: &db::DB, user: &users::Data) -> Result<AuthnWithStored, diesel::result::Error> {
        match self {
            Self::Password(p) => db.find_pw_hash_by_user(user).map(move |d| AuthnWithStored::Password(p, d)),
        }
    }
    fn create(&self, db: &db::DB, user: &users::Data, creator_id: uuid::Uuid, key: &<ARGON2D as A>::Key) -> Result<AuthnWithStored, auth::Error> {
        Ok(match self {
            Self::Password(p) => {
                let msg = &ARGON2D_MSG::new_default_hash_len(
                    p.password.as_bytes().to_vec(),
                    None,
                );
                let generated_salt = msg.salt();
                let pw_hash = <ARGON2D as HashA>::sign(
                    msg,
                    key,
                );
                AuthnWithStored::Password(p, db.create_pw_hash(credentials::pw::New {
                    created_by: creator_id,
                    updated_by: creator_id,
                    user_id: user.id,
                    hash: base64::encode(pw_hash.as_slice()).as_str(),
                    salt: base64::encode(generated_salt).as_str(),
                })?)
            },
        })
    }
}

