//! Groups all the static pages together.

use maud::Markup;
use page_client;
use rocket::Route;

mod contacts;
mod links;
mod projects;
mod resume;

/// Returns the "index" page, aka the home page of the website.
///
/// This simply calls [`page_client::index()`] from [`page_client`].
#[get("/")]
fn get_index() -> Markup {
    page_client::home::index()
}

/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`].
pub fn routes() -> Vec<Route> {
    routes![
        get_index,
        resume::get,
        links::get,
        contacts::get,
        projects::get,
        projects::project::get,
    ]
}
