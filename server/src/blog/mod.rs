pub mod db;
pub use db::DB;
pub mod accounts;
pub mod auth;
pub mod credentials;
pub mod permissions;
pub mod posts;
pub mod login;

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
        login::get,
        login::post,
        login::delete,
        credentials::pws::post,
        credentials::pws::pw::patch,
        credentials::pws::pw::delete,
        permissions::post,
        permissions::delete,
        permissions::permission::get,
        permissions::permission::delete,
    ]
}

