pub mod data;

use std::sync::Arc;

use rocket::{
    response::status,
    http::Status,
    State,
};
use rocket_contrib::{
    json::Json,
    uuid::Uuid as RUuid,
};

use crate::{
    uuid_conv::FromRUuid,
    crypto::{
        KeyStore,
        algo::hash::argon2::d::Algo as ARGON2D,
    },
    blog::{
        db,
        auth,
        credentials::data::SavableCredential,
    },
};

pub mod pws {
    use super::*;

    #[post("/credentials/pws", format = "json", data = "<to_create>")]
    pub fn post(
        db: db::DB,
        credentials: auth::UnverifiedPermissionsCredential,
        pw_key_store: State<Arc<KeyStore<ARGON2D>>>,
        to_create: Json<data::Password>,
    ) -> status::Custom<()> {
        let curr_and_last = pw_key_store.curr_and_last();
        let to_create = data::PasswordWithBackingInfo {
            db: &db,
            credentials: &credentials,
            argon2d_key: match &curr_and_last {
                Ok(k) => &k.curr,
                Err(_) => return status::Custom(Status::InternalServerError, ()),
            },
            pw: &to_create,
        };
        match to_create.convert_and_save_with_credentials() {
            Ok(()) => status::Custom(Status::Ok, ()),
            Err(()) => status::Custom(Status::InternalServerError, ()),
        }
    }

    pub mod pw {
        use super::*;

        #[patch("/credentials/pws/<id>", format = "json", data = "<update>")]
        pub fn patch(
            db: db::DB,
            pw_key_store: State<Arc<KeyStore<ARGON2D>>>,
            id: RUuid,
            credentials: auth::UnverifiedPermissionsCredential,
            update: Json<data::Password>,
        ) -> status::Custom<()> {
            let curr_and_last = pw_key_store.curr_and_last();
            // TODO fix this -- this utilizes a field in update, should get rid of that
            let to_create = data::PasswordWithBackingInfo {
                db: &db,
                credentials: &credentials,
                argon2d_key: match &curr_and_last {
                    Ok(k) => &k.curr,
                    Err(_) => return status::Custom(Status::InternalServerError, ()),
                },
                pw: &update,
            };
            match to_create.convert_and_update_with_credentials() {
                Ok(()) => status::Custom(Status::Ok, ()),
                Err(()) => status::Custom(Status::InternalServerError, ()),
            }
        }
        #[delete("/credentials/pws/<id>")]
        pub fn delete(
            db: db::DB,
            id: RUuid,
            credentials: auth::UnverifiedPermissionsCredential,
        ) -> status::Custom<()> {
            // TODO check credentials
            match db.delete_pw_by_id(uuid::Uuid::from_ruuid(id)) {
                Ok(_) => status::Custom(Status::Ok, ()),
                Err(_) => status::Custom(Status::InternalServerError, ()),
            }
        }
    }
}

