use crate::pages;
use rocket::Route;
use maud::{html, Markup};

mod links;
mod contacts;
mod resume;
mod projects;
pub mod blog;

#[get("/")]
fn get_index() -> Markup {
    let msg = "msg";
    pages::basic_page(html! {
        p { (msg) }
    }, None)
}

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

