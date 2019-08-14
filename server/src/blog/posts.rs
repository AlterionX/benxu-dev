use rocket_contrib::json::Json;
use crate::blog::DB;
use blog_db::models::*;

#[post("/posts")]
pub fn post(db: DB) -> Result<Json<Post>, &'static str> {
    let post = NewPost {
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
    use rocket_contrib::json::Json;
    use crate::blog::db::DB;
    use blog_db::models::*;

    #[get("/posts/<id>")]
    pub fn get<'a>(id: i32, db: DB) -> Result<Json<Post>, String> {
        if let Ok(post) = db.find_post_with_id(id) {
            Ok(Json(post))
        } else {
            Err(format!("Failed to find match for {}.", id))
        }
    }
    #[patch("/posts/<id>", data = "<update_post>")]
    pub fn patch(id: i32, update_post: Json<UpdatePost>, db: DB) -> status::Custom<()> {
        let update = &*update_post;
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
    pub fn delete(id: i32, db: DB) -> status::Custom<()> {
        if let Ok(post) = db.delete_post_with_id(id) {
            Json(post);
        } else {
            format!("Failed to find match for {}.", id);
        }
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
}
