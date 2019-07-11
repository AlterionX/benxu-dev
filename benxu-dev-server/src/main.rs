#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate maud;
extern crate benxu_dev_page_client;

use benxu_dev_page_client as pages;
use rocket_contrib::serve::StaticFiles;
mod routing;

fn main() {
    rocket::ignite().mount(
        "/",
        routing::routes(),
    ).mount(
        "/assets",
        StaticFiles::from("/"),
    ).mount(
        "/blog",
        routing::blog::routes(),
    ).launch();
}

