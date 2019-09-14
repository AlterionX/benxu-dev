use rocket_contrib::json::Json;
use crate::blog::DB;
use blog_db::models::*;

#[post("/posts")]
pub fn post(db: DB) -> Result<Json<posts::Data>, &'static str> {
    let post = posts::New {
        title: "Hi",
        body: "World!",
    };
    if let Ok(saved_post) = db.insert_post(post) {
        Ok(Json(saved_post))
    } else {
        Err("Post failed to save.")
    }
}

pub mod post {
    use rocket::{response::status, http::Status};
    use rocket_contrib::{json::Json, uuid::Uuid as RUuid};
    use crate::blog::{db::DB, auth};
    use blog_db::models::*;

    enum ConvError {
        Parse(uuid::ParseError),
        Version(usize),
    }
    impl From<uuid::ParseError> for ConvError {
        fn from(parse_err: uuid::ParseError) -> Self {
            Self::Parse(parse_err)
        }
    }
    impl From<usize> for ConvError {
        fn from(version_num: usize) -> Self {
            Self::Version(version_num)
        }
    }
    impl From<ConvError> for status::Custom<()> {
        fn from(e: ConvError) -> Self {
            status::Custom(Status::BadRequest, ())
        }
    }

    fn conv_ruuid(uuid: RUuid) -> Result<uuid::Uuid, ConvError> {
        let version_num = uuid.get_version_num();
        if version_num != 4 {
            Err(version_num)?
        } else {
            Ok(uuid::Uuid::from_bytes(uuid.as_bytes())?)
        }
    }

    #[get("/posts/<id>")]
    pub fn get(id: RUuid, db: DB) -> Result<Json<posts::Data>, status::Custom<()>> {
        let id = conv_ruuid(id)?;
        // TODO eventually consider error messages for different DB failures
        db.find_post_with_id(id)
            .map(|post| Json(post))
            .map_err(|_| status::Custom(Status::BadRequest, ()))
    }
    #[patch("/posts/<id>", data = "<update>")]
    pub fn patch(
        id: RUuid,
        update: Json<posts::Changed>,
        db: DB,
        _auth: auth::Credentials<auth::perms::CanEdit>,
    ) -> status::Custom<()> {
        let id = if let Ok(id) = conv_ruuid(id) {
            id
        } else {
            return status::Custom(Status::BadRequest, ());
        };
        let update = &*update;
        if let Ok(rows_updated) = db.update_post_with_id(id, update) {
            if rows_updated == 0 {
                status::Custom(Status::new(404, "Post not found"), ())
            } else {
                status::Custom(Status::new(200, "Success"), ())
            }
        } else {
            status::Custom(Status::new(501, "Database error"), ())
        }
    }
    #[delete("/posts/<id>")]
    pub fn delete(
        id: RUuid,
        db: DB,
        _auth: auth::Credentials<auth::perms::CanDelete>,
    ) -> status::Custom<()> {
        let id = if let Ok(id) = conv_ruuid(id) {
            id
        } else {
            return status::Custom(Status::BadRequest, ());
        };
        if let Ok(post) = db.delete_post_with_id(id) {
            Json(post);
        } else {
            format!("Failed to find match for {}.", id);
        }
        status::Custom(Status::NotImplemented, ())
    }
    #[post("/posts/<id>/publish")]
    pub fn publish(
        id: RUuid,
        db: DB,
        auth: auth::Credentials<auth::perms::CanPublish>,
    ) -> status::Custom<()> {
        let id = if let Ok(id) = conv_ruuid(id) {
            id
        } else {
            return status::Custom(Status::BadRequest, ());
        };
        return match db.publish_post_with_id(id, posts::Publishing::new(auth.to_user_id())) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(_) => status::Custom(Status::BadRequest, ()),
        }
    }
    #[post("/posts/<id>/archive")]
    pub fn archive(
        id: RUuid,
        db: DB,
        auth: auth::Credentials<auth::perms::CanArchive>,
    ) -> status::Custom<()> {
        let id = if let Ok(id) = conv_ruuid(id) {
            id
        } else {
            return status::Custom(Status::BadRequest, ());
        };
        return match db.archive_post_with_id(id, posts::Archival::new(auth.to_user_id())) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(_) => status::Custom(Status::BadRequest, ()),
        }
    }
}
