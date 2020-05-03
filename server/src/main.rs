#![feature(proc_macro_hygiene, type_ascription, decl_macro, try_trait)]

//! Server crate for marshalling and unmarshalling information between the blog-db and blog-client
//! crates as well as serving a set of static pages.
//!
//! This utilizes the following path structure:
//! - `/` -> Home pagea and other static pages are attached here. See the [`fixed`] module for more information.
//! - `/blog/*` -> Blog related information. See the [`blog`] module for more information.
//! - `/public/*` -> All static resources for the site. These are served from `./public/` using the
//!   [`StaticFiles`] module.

#[macro_use]
extern crate rocket;

use crypto;
use log::*;
use rocket_contrib::serve::StaticFiles;
use std::{path::Path, sync::Arc};
use tap::*;

mod cfg;

mod urls;
mod util;

use crate::{
    urls::{blog_api_routes, blog_spa_routes, fixed_routes},
    util::blog::DB as BlogDB,
};

mod shared_html {
    pub fn logo_markup() -> Option<page_client::data::Logo<'static>> {
        Some(page_client::data::Logo {
            src: "/public/img/branding.svg",
            href: Some("/"),
        })
    }
}

/// A struct to ensure correct initialization of the server.
struct Server {
    /// Path to configuration file as per dotenv.
    _env_path: std::path::PathBuf,
    /// SodiumOxide crypto library initialization -- used as a reminder.
    _sodiumoxide_init: (),
    /// Key store + key rotation for the PASETO v2 local tokens used for authz.
    _paseto_key: crypto::KeyRotator<cfg::TokenAlgo>,
    /// Key store for passwords secret keys.
    _local_loaded_key: Arc<crypto::StableKeyStore<cfg::PWAlgo>>,
    /// The Rocket instance managing all handlers and data routing.
    rocket: Option<rocket::Rocket>,
}
impl Server {
    /// Initializes and launches all required components that manage state in the server.
    fn new() -> Self {
        // Initializing environment variables.
        let env_path = {
            info!("Initialize multithreaded crypto crate.");
            let path = dotenv::dotenv().expect("WARNING could not load env vars.");
            info!("Crypto crate initialized.");
            path
        };
        let public_path = {
            info!("Locating static files directory...");
            // TODO dynamically load public directory path from config file.
            let path_str = cfg::PUBLIC_DIRECTORY;
            let path = Path::new(cfg::PUBLIC_DIRECTORY)
                .canonicalize()
                .tap_ok(|p| info!("Serving static files from `{}`.", p.display()))
                .tap_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        error!("Could not find path `{}` in file system", path_str)
                    }
                    _ => error!("Unhandled IO error for path `{}`:\n{:?}", path_str, e),
                })
                .expect("a proper path.");
            info!("Crypto crate initialized.");
            path
        };
        let pw_secret_path = {
            info!("Locating password hashing secret...");
            let path = cfg::pw_secret().expect("A proper path to a proper file.");
            info!("Password secret located.");
            path // TODO unwrap + actually load this file
        };
        // Initializing cryptographic system.
        let sodiumoxide_init = {
            info!("Initialize multithreaded crypto crate.");
            let res = crypto::multithread_init().tap_err(|_| {
                error!(
                    "Could not initialize crate `crypto` for multithreaded use. Will panic later."
                )
            });
            info!("Crypto crate initialized.");
            res
        };
        let paseto_key = {
            info!("Initializing token cryptographic key rotation...");
            let rotator = cfg::init_token_key();
            info!("Token cryptographic key rotation initialized.");
            rotator
        };
        let local_loaded_key = {
            info!("Initializing password secret key...");
            let static_key_store = Arc::new(cfg::init_pw_secret(&pw_secret_path));
            info!("Password secret key initialized.");
            static_key_store
        };
        // Initializing rocket and attaching all the things.
        let rocket = {
            info!("Prepping Rocket...");
            let rocket = rocket::ignite()
                .mount(cfg::STATIC_ROOT, fixed_routes())
                .mount(cfg::PUBLIC_ROOT, StaticFiles::from(public_path))
                .attach(BlogDB::fairing())
                .manage(Arc::clone(&local_loaded_key))
                .manage(paseto_key.get_key_fixture())
                .mount(cfg::BLOG_API_ROOT, blog_api_routes())
                .mount(cfg::BLOG_SPA_ROOT, blog_spa_routes());
            info!("Rocket ready for launch!");
            rocket
        };
        Server {
            _env_path: env_path,

            _sodiumoxide_init: sodiumoxide_init.unwrap(),
            _paseto_key: paseto_key,
            _local_loaded_key: local_loaded_key,

            rocket: Some(rocket),
        }
    }
    /// Passes off execution to [`Rocket`](rocket::Rocket).
    ///
    /// NOTE: This behavior may change in an async version of [`Rocket`](rocket::Rocket).
    fn start(&mut self) -> Option<rocket::error::LaunchError> {
        self.rocket.take().map(|r| r.launch())
    }
}

/// Initializes server and listens for errors that occur after launching rocket.
fn main() {
    simple_logger::init_with_level(log::Level::Trace)
        .expect("No problems initializing simple_logger.");
    info!("Initializing server...");
    let mut server = Server::new();
    info!("Server initialized!");
    info!("Launching rocket into the ether (aka, passing control to Rocket)...");
    match server.start() {
        None => warn!("Rocket has already been launched somehow!"),
        Some(e) => error!(
            "Rocket has terminated with error {:?}. Server will now shutdown.",
            e
        ),
    }
}

// TODO tests?
