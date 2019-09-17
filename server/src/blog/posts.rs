//! Handlers and functions for managing posts.

use rocket::http::Status;
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use crate::{
    blog::{auth, DB},
    uuid_conv::FromRUuid,
};
use blog_db::models::*;

/// Handler for posting a post to the database. Requires user to be logged in and have the
/// [`CanPost`](crate::blog::auth::perms::CanPost) permission.
#[post("/posts", format = "json", data = "<post>")]
pub fn post(
    db: DB,
    credentials: auth::Credentials<auth::perms::CanPost>,
    post: Json<posts::NewNoMeta>,
) -> Result<Json<posts::Data>, Status> {
    let post = post.into_inner();
    db.insert_post((&post, credentials.user_id()))
        .map(|p| Json(p))
        .map_err(|_e| {
            // TODO log error
            Status::InternalServerError
        })
}

/// Handlers and functions for managing or retrieving individual posts.
pub mod post {
    use super::*;

    /// Map a rather common diesel error to it's corresponding http [`Status`](rocket::http::Status).
    ///
    /// If there is exactly one result, it is [`Ok`]. If there are no results OR the error is the
    /// [`Error::NotFound`](diesel::result::Error::NotFound) error,
    /// [`Status::NotFound`](rocket::http::Status::NotFound) is returned.
    ///
    /// Otherwise, we return
    /// [`Status::InternalServerError`](rocket::http::Status::InternalServerError).
    fn map_to_status(res: Result<usize, diesel::result::Error>) -> Status {
        match res {
            Ok(1) => Status::Ok,
            Ok(0) | Err(diesel::result::Error::NotFound) => Status::NotFound,
            _ => Status::InternalServerError,
        }
    }

    /// Handler for retrieving a post with a specific id. No permissions needed.
    #[get("/posts/<id>")]
    pub fn get(db: DB, id: RUuid) -> Result<Json<posts::Data>, Status> {
        let id = uuid::Uuid::from_ruuid(id);
        // TODO eventually consider error messages for different DB failures
        db.find_post_with_id(id)
            .map(|post| Json(post))
            .map_err(|_| Status::BadRequest)
    }
    /// Handler for editing a post with a specific id. Requires user to be logged in and have the
    /// [`CanPost`](crate::blog::auth::perms::CanEdit) permission.
    #[patch("/posts/<id>", data = "<update>")]
    pub fn patch(
        db: DB,
        id: RUuid,
        update: Json<posts::Changed>,
        _editor: auth::Credentials<auth::perms::CanEdit>,
    ) -> Status {
        let id = uuid::Uuid::from_ruuid(id);
        map_to_status(db.update_post_with_id(id, &update.into_inner()))
    }
    /// Handler for deleting a post with a specific id. Requires user to be logged in and have
    /// the [`CanDelete`](crate::blog::auth::perms::CanDelete) permission.
    #[delete("/posts/<id>")]
    pub fn delete(id: RUuid, db: DB, deleter: auth::Credentials<auth::perms::CanDelete>) -> Status {
        let id = uuid::Uuid::from_ruuid(id);
        map_to_status(db.delete_post_with_id(id, &posts::Deletion::new(deleter.user_id())))
    }
    /// Handler for publishing a post with a specific id. Requires user to be logged in and have
    /// the [`CanPublish`](crate::blog::auth::perms::CanPublish) permission.
    #[post("/posts/<id>/publish")]
    pub fn publish(
        id: RUuid,
        db: DB,
        publisher: auth::Credentials<auth::perms::CanPublish>,
    ) -> Status {
        let id = uuid::Uuid::from_ruuid(id);
        let publisher = publisher.user_id();
        map_to_status(db.publish_post_with_id(id, posts::Publishing::new(publisher)))
    }
    /// Handler for publishing a post with a specific id. Requires user to be logged in and have
    /// the [`CanPublish`](crate::blog::auth::perms::CanPublish) permission.
    #[post("/posts/<id>/archive")]
    pub fn archive(
        id: RUuid,
        db: DB,
        archiver: auth::Credentials<auth::perms::CanArchive>,
    ) -> Status {
        let id = uuid::Uuid::from_ruuid(id);
        map_to_status(db.archive_post_with_id(id, posts::Archival::new(archiver.user_id())))
    }
}
