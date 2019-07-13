#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate maud;
extern crate benxu_dev_page_client;

use benxu_dev_page_client as pages;
use std::path::Path;
use rocket_contrib::serve::StaticFiles;

mod routing;

fn main() {
    let public_path = Path::new("./public").canonicalize().unwrap();
    println!("Serving files from {:?}", public_path);
    rocket::ignite().mount(
        "/",
        routing::routes(),
    ).mount(
        "/public",
        StaticFiles::from(public_path),
    ).mount(
        "/blog",
        routing::blog::routes(),
    ).launch();
}

