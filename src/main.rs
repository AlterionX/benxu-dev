#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate maud;
extern crate chrono;

mod routing;
mod pages;


fn main() {
    rocket::ignite().mount(
        "/",
        routing::routes(),
    ).mount(
        "/blog",
        routing::blog::routes(),
    ).launch();
}

