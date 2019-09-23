//! Marshalls the data between the [`blog_client`](../blog_client) and [`blog_db`](../blog_db) as well as performing
//! authentication and authorization through the [`auth`](crate::blog::auth) module.

pub mod db;
pub use db::DB;
pub mod accounts;
pub mod auth;
pub mod credentials;
pub mod login;
pub mod permissions;
pub mod posts;

use rocket::Route;
use maud::Markup;

/// Handler for serving the primary web app.
#[get("/<_path..>")]
pub fn get(_path: Option<rocket::http::uri::Segments>) -> Markup {
    page_client::blog::index()
}
/// Handler for serving the primary web app for when there is no path.
#[get("/")]
pub fn get_unadorned() -> Markup {
    page_client::blog::index()
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

/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`]. Used for the
/// SPA endpoints.
pub fn spa_routes() -> Vec<Route> {
    routes![get, get_unadorned]
}
/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`]. Used for the
/// api endpoints.
pub fn api_routes() -> Vec<Route> {
    routes![
        posts::get,
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
