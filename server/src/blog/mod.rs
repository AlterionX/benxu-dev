//! Marshalls the data between the [`blog_client`](../blog_client) and [`blog_db`](../blog_db) as well as performing
//! authentication and authorization through the [`auth`] module.

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
    http::Status
};

/// Handler for serving the primary web app, to be implemented.
#[get("/")]
pub fn get() -> Status {
    Status::NotImplemented
}

/// Handlers, functions, structs for marshalling editor data and retrieving the webpage.
pub mod editor {
    use rocket::http::Status;
    /// Handler for serving the editor page, to be implemented.
    #[get("/editor")]
    pub fn get() -> Status {
        Status::NotImplemented
    }
}

/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`].
pub fn routes() -> Vec<Route> {
    routes![
        get, // blog front page
        posts::post,
        posts::post::get,
        posts::post::patch,
        posts::post::delete,
        posts::post::publish,
        posts::post::archive,
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

