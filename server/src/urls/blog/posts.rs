//! Handlers and functions for managing posts.

use rocket::http::{RawStr, Status};
use rocket_contrib::{json::Json, uuid::Uuid as RUuid};

use chrono::DateTime;
use tap::*;

use crate::util::{
    auth,
    blog::{
        db::{self, PostQuery},
        DB,
    },
    uuid_compat::ruuid_to_uuid,
};
use blog_db::models::*;

/// Handler for getting posts with criteria.
#[get(
    "/posts?<offset>&<lim>&<start_time>&<stop_time>&<ord_criteria>&<ord>",
    format = "json"
)]
pub fn get(
    db: DB,
    start_time: Option<&RawStr>,
    stop_time: Option<&RawStr>,
    offset: Option<usize>,
    lim: Option<usize>,
    ord_criteria: Option<db::OrderingField>,
    ord: Option<db::SortOrdering>,
) -> Result<Json<Vec<posts::BasicData>>, Status> {
    let ord_criteria = ord_criteria.unwrap_or(db::OrderingField::Date);
    let ord = ord.unwrap_or_else(|| match ord_criteria {
        db::OrderingField::Date => db::SortOrdering::Descending,
        db::OrderingField::AlphabeticalTitle => db::SortOrdering::Ascending,
    });
    let num_passed = [
        start_time.is_some(),
        stop_time.is_some(),
        offset.is_some(),
        lim.is_some(),
    ]
    .iter()
    .filter(|b| **b)
    .count();
    if num_passed != 2 {
        log::error!("Post search request made with more or less than 2 restrictions.");
        Err(Status::BadRequest)
    } else if let (Some(start_time), Some(stop_time)) = (start_time, stop_time) {
        get_by_date_range(db, start_time, stop_time, ord_criteria, ord)
    } else if let (Some(lim), Some(offset)) = (lim, offset) {
        get_by_limit_and_offset(db, offset, lim, ord_criteria, ord)
    } else {
        log::error!("Post search request made with a mismatched pair of restrictions.");
        Err(Status::BadRequest)
    }
}
/// Handler for getting posts between two times.
pub fn get_by_date_range(
    db: DB,
    start_time: &RawStr,
    stop_time: &RawStr,
    ord_criteria: db::OrderingField,
    ord: db::SortOrdering,
) -> Result<Json<Vec<posts::BasicData>>, Status> {
    let start_time = start_time
        .percent_decode()
        .as_ref()
        .map(|c| &**c)
        .map(DateTime::parse_from_rfc3339)
        .map_err(|_| Status::BadRequest)?
        .map_err(|_| Status::BadRequest)?;
    let stop_time = stop_time
        .percent_decode()
        .as_ref()
        .map(|c| &**c)
        .map(DateTime::parse_from_rfc3339)
        .map_err(|_| Status::BadRequest)?
        .map_err(|_| Status::BadRequest)?;
    let max_posts = 500;
    db.find_posts_with_post_listing_conditions(db::PostListing::Date {
        start: start_time.into(),
        stop: stop_time.into(),
        order_by: ord_criteria,
        ord,
        limit: max_posts,
    })
    .tap_err(|e| log::error!("Failed to find posts by date range due to error {:?}.", e))
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

/// Handler for getting posts with an offset and a limit.
#[get("/posts?<offset>&<lim>&<ord_criteria>&<ord>", format = "json")]
pub fn get_by_limit_and_offset(
    db: DB,
    offset: usize,
    lim: usize,
    ord_criteria: db::OrderingField,
    ord: db::SortOrdering,
) -> Result<Json<Vec<posts::BasicData>>, Status> {
    db.find_posts_with_post_listing_conditions(db::PostListing::LimAndOffset {
        offset,
        lim: std::cmp::min(lim, 500),
        order_by: ord_criteria,
        ord,
    })
    .tap_err(|e| log::error!("Failed to find posts due to error {:?}.", e))
    .map(Json)
    .map_err(|_| Status::InternalServerError)
}

/// Handler for posting a post to the database. Requires user to be logged in and have the
/// [`Post`](crate::blog::auth::caps::Post) capability.
#[post("/posts", format = "json", data = "<post>")]
pub fn post(
    db: DB,
    capabilities: auth::Capabilities<auth::caps::Post>,
    post: Json<posts::NewNoMeta>,
) -> Result<Json<posts::Data>, Status> {
    let post = post.into_inner();
    db.insert_post((&post, capabilities.user_id()))
        .tap_err(|e| log::error!("Failed to create new post due to error {:?}.", e))
        .map(Json)
        .map_err(|_| Status::InternalServerError)
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

    /// Handler for retrieving a post with a specific id. No capabilities needed.
    #[get("/posts/<id>")]
    pub fn get(db: DB, id: RUuid) -> Result<Json<posts::Data>, Status> {
        let id = ruuid_to_uuid(id);
        db.find_post_with_id(id)
            .tap_err(|e| log::error!("Failed to retrieve post {:?} due to DB error {:?}.", id, e))
            .map(Json)
            .map_err(|_| Status::BadRequest)
    }
    /// Handler for editing a post with a specific id. Requires user to be logged in and have the
    /// [`Post`](crate::blog::auth::caps::Edit) capability.
    #[patch("/posts/<id>", data = "<update>")]
    pub fn patch(
        id: RUuid,
        update: Json<posts::Changed>,
        _editor: auth::Capabilities<auth::caps::Edit>,
        db: DB,
    ) -> Status {
        let id = ruuid_to_uuid(id);
        let res = db.update_post_with_id(id, &update.into_inner());
        map_to_status(res)
    }
    /// Handler for deleting a post with a specific id. Requires user to be logged in and have
    /// the [`Delete`](crate::blog::auth::caps::Delete) capability.
    #[delete("/posts/<id>")]
    pub fn delete(id: RUuid, db: DB, deleter: auth::Capabilities<auth::caps::Delete>) -> Status {
        let id = ruuid_to_uuid(id);
        let deletion_update = posts::Deletion::new(deleter.user_id());
        let req = db.delete_post_with_id(id, &deletion_update)
            .tap_err(|e| log::error!("Failed to delete post {:?} due to error {:?}.", id, e));
        map_to_status(req)
    }
    /// Handler for publishing a post with a specific id. Requires user to be logged in and have
    /// the [`Publish`](crate::blog::auth::caps::Publish) capability.
    #[post("/posts/<id>/publish", data = "<update>")]
    pub fn publish(
        id: RUuid,
        db: DB,
        update: Option<Json<posts::Changed>>,
        publisher: auth::Capabilities<auth::caps::Publish>,
    ) -> Status {
        let id = ruuid_to_uuid(id);
        if let Some(update) = update {
            let update = update.into_inner();
            let changed_credential = publisher.clone().change_level::<auth::caps::Edit>();
            if let Ok(_) = changed_credential {
                let status = map_to_status(db.update_post_with_id(id, &update));
                if status != Status::Ok {
                    return status;
                }
            } else {
                return Status::Unauthorized;
            }
        }
        let publisher = publisher.user_id();
        map_to_status(db.publish_post_with_id(id, posts::Publishing::new(publisher)))
    }
    /// Handler for publishing a post with a specific id. Requires user to be logged in and have
    /// the [`Publish`](crate::blog::auth::caps::Publish) capability.
    #[post("/posts/<id>/archive")]
    pub fn archive(
        id: RUuid,
        db: DB,
        archiver: auth::Capabilities<auth::caps::Archive>,
    ) -> Status {
        let id = ruuid_to_uuid(id);
        map_to_status(db.archive_post_with_id(id, posts::Archival::new(archiver.user_id())))
    }
}
