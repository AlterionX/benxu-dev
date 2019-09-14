#![feature(const_str_as_bytes, proc_macro_hygiene, type_ascription, decl_macro, try_trait)]

#[macro_use] extern crate rocket;

use std::{
    path::Path,
    sync::Arc,
};
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket_contrib::serve::StaticFiles;
use page_client as pages;

mod uuid_conv;

mod crypto;
mod encoding;
mod fixed;
mod blog;

type DefaultAlgo = crypto::algo::cipher::plaintext::PlainTextAlgo;

fn main() {
    // initialize + prep all the things
    sodiumoxide::init().unwrap();
    dotenv().expect("All systems nominal");
    let public_path = Path::new("./public").canonicalize().unwrap();
    rocket::ignite().mount(
        "/",
        fixed::routes(),
    ).mount(
        "/public",
        StaticFiles::from(public_path),
    ).attach(
        blog::DB::fairing(),
    ).attach(
        AdHoc::on_attach("Crypto: Key Rotation Mechanism", |rocket| {
            // TODO when rocket 0.5 lands, graceful shutdown
            let key_rotator = Box::new(crypto::KeyRotator::<
                crate::DefaultAlgo,
            >::init(
                DefaultAlgo {},
                None,
            ));
            let key_store = Arc::clone(&key_rotator.key_store);
            let rocket = rocket.manage(key_store);
            Box::leak(key_rotator);
            Ok(rocket)
        }),
    ).mount(
        "/blog",
        blog::routes(),
    ).launch();
}

