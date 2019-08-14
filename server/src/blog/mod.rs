pub mod db;
pub use db::DB;
pub mod posts;
pub mod auth;

use rocket::{
    Route,
    response::status,
    http::Status
};

#[get("/")]
pub fn get() -> status::Custom<()> {
    status::Custom(Status::new(501, "Not yet implemented"), ())
}

pub mod editor {
    #[get("/editor")]
    pub fn get() -> &'static str {
        "editor, eventually"
    }
}

pub mod accounts {
    use rocket::{response::status, http::Status};
    #[post("/accounts")]
    pub fn post() -> status::Custom<()> {
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
    pub mod account {
        use rocket::{response::status, http::{RawStr, Status}};
        #[get("/accounts/<id>")]
        pub fn get(id: &RawStr) {
        }
        #[patch("/accounts/<id>")]
        pub fn patch(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
        #[delete("/accounts/<id>")]
        pub fn delete(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        get, // blog front page
        posts::post,
        posts::post::get,
        posts::post::patch,
        posts::post::delete,
        editor::get,
        accounts::post,
        accounts::account::get,
        accounts::account::patch,
        accounts::account::delete,
        auth::get,
        auth::post,
        auth::delete,
    ]
}
