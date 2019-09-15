pub mod error;

use rocket::{response::status, http::Status};
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use blog_db::models::*;
use crate::{
    blog::{
        DB,
        auth,
    },
    uuid_conv::FromRUuid,
};

#[post("/posts", format = "json", data = "<post>")]
pub fn post(db: DB, credentials: auth::Credentials<auth::perms::CanPost>, post: Json<posts::NewNoMeta>) -> Result<Json<posts::Data>, &'static str> {
    let post = post.into_inner();
    if let Ok(saved_post) = db.insert_post((&post, credentials.user_id()).into()) {
        Ok(Json(saved_post))
    } else {
        Err("Post failed to save.")
    }
}

pub mod post {
    use super::*;

    #[get("/posts/<id>")]
    pub fn get(
        id: RUuid,
        db: DB,
    ) -> Result<Json<posts::Data>, status::Custom<()>> {
        let id = uuid::Uuid::from_ruuid(id);
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
        _editor: auth::Credentials<auth::perms::CanEdit>,
    ) -> status::Custom<()> {
        let id = uuid::Uuid::from_ruuid(id);
        return match db.update_post_with_id(id, &update.into_inner()) {
            Ok(1) => status::Custom(Status::Ok, ()),
            Ok(0) | Err(diesel::result::Error::NotFound) => status::Custom(Status::InternalServerError, ()),
            _ => status::Custom(Status::InternalServerError, ()),
        }
    }
    #[delete("/posts/<id>")]
    pub fn delete(
        id: RUuid,
        db: DB,
        deleter: auth::Credentials<auth::perms::CanDelete>,
    ) -> status::Custom<()> {
        let id = uuid::Uuid::from_ruuid(id);
        return match db.delete_post_with_id(id, &posts::Deletion::new(deleter.user_id())) {
            Ok(1) => status::Custom(Status::Ok, ()),
            Ok(0) | Err(diesel::result::Error::NotFound) => status::Custom(Status::InternalServerError, ()),
            _ => status::Custom(Status::InternalServerError, ()),
        }
    }
    #[post("/posts/<id>/publish")]
    pub fn publish(
        id: RUuid,
        db: DB,
        auth: auth::Credentials<auth::perms::CanPublish>,
    ) -> status::Custom<()> {
        let id = uuid::Uuid::from_ruuid(id);
        let publisher = auth.user_id();
        return match db.publish_post_with_id(id, posts::Publishing::new(publisher)) {
            Ok(1) => status::Custom(Status::Ok, ()),
            Ok(0) | Err(diesel::result::Error::NotFound) => status::Custom(Status::InternalServerError, ()),
            _ => status::Custom(Status::InternalServerError, ()),
        }
    }
    #[post("/posts/<id>/archive")]
    pub fn archive(
        id: RUuid,
        db: DB,
        archiver: auth::Credentials<auth::perms::CanArchive>,
    ) -> status::Custom<()> {
        let id = uuid::Uuid::from_ruuid(id);
        return match db.archive_post_with_id(id, posts::Archival::new(archiver.user_id())) {
            Ok(1) => status::Custom(Status::Ok, ()),
            Ok(0) | Err(diesel::result::Error::NotFound) => status::Custom(Status::InternalServerError, ()),
            _ => status::Custom(Status::InternalServerError, ()),
        }
    }
}

