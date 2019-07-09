#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

mod links;
mod contacts;
mod resume;
mod projects;
mod blog;

#[get("/")]
fn get_index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount(
        "/",
        routes![
            get_index,
            resume::get,
            links::get,
            contacts::get,
            projects::get,
            projects::project::get,
        ],
    ).mount(
        "/blog",
        blog::routes(),
    ).launch();
}

