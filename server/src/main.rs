#![feature(const_str_as_bytes, proc_macro_hygiene, type_ascription, decl_macro, try_trait)]

#[macro_use] extern crate rocket;

use std::{
    path::Path,
    sync::Arc,
};
use rocket::fairing::AdHoc;
use rocket_contrib::serve::StaticFiles;
use page_client as pages;
use crypto;

mod uuid_conv;

mod fixed;
mod blog;

type PWAlgo = crypto::algo::hash::argon2::d::Algo;
type TokenAlgo = crypto::token::paseto::v2::local::Algo;
type DefaultAlgo = crypto::algo::cipher::plaintext::PlainTextAlgo;

/// A struct to ensure correct initialization of the server.
struct Server {
    /// Path to configuration file as per dotenv.
    _env_path: std::path::PathBuf,
    /// SodiumOxide crypto library initialization -- used as a reminder.
    _sodiumoxide_init: (),
    /// Key store + key rotation for the PASETO v2 local tokens used for authz.
    _paseto_key: crypto::KeyRotator<TokenAlgo>,
    /// Key store for passwords secret keys.
    _local_loaded_key: crypto::key_rotation::StaticKeyStore<PWAlgo>,
    /// The Rocket instance managing all handlers and data routing.
    rocket: rocket::Rocket,
}
impl Server {
    const PW_SECRET_KEY_DEFAULT_PATH: &'static str = "./.pw_secret";
    const PW_SECRET_KEY_ENV_VAR_NAME: &'static str = "BENXU_DEV_PW_HASH";

    const STATIC_ROOT: &'static str = "/";
    const PUBLIC_ROOT: &'static str = "/public";
    const BLOG_ROOT: &'static str = "/blog";

    fn pw_secret() -> std::path::PathBuf {
        let env_var_res = std::env::var(Self::PW_SECRET_KEY_ENV_VAR_NAME);
        let pw_file = env_var_res
            .as_ref()
            .map(|s| s.as_ref())
            .unwrap_or(Self::PW_SECRET_KEY_DEFAULT_PATH);
        let pw_file = Path::new(pw_file)
            .canonicalize()
            .unwrap();
        pw_file
    }
    fn new() -> Self {
        // initialize + prep all the things
        let sodiumoxide_init = sodiumoxide::init().unwrap();
        let env_path = dotenv::dotenv().expect("WARNING could not load env vars.");
        let public_path = Path::new("./public").canonicalize().unwrap();
                    let pw_secret = Self::pw_secret();
                    let _paseto_key = crypto::KeyRotator::<
                        crate::TokenAlgo,
                    >::init(
                        TokenAlgo {},
                        None,
                    );
                    let _local_loaded_key = crypto::key_rotation::StaticKeyStore::<
                        crate::PWAlgo,
                    >::new(PWAlgo::default(), <<PWAlgo as crypto::algo::Algo>::Key as crypto::algo::SafeGenerateKey>::generate(&()));
        let rocket = rocket::ignite()
            .mount(Self::STATIC_ROOT, fixed::routes())
            .mount(Self::PUBLIC_ROOT, StaticFiles::from(public_path))
            .attach(blog::DB::fairing())
            .attach(
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
            )
            .mount(Self::BLOG_ROOT, blog::routes());
        Server {
            _env_path: env_path,

            _sodiumoxide_init: sodiumoxide_init,
            _paseto_key: _paseto_key,
            _local_loaded_key: _local_loaded_key,

            rocket: rocket,
        }
    }
}

fn main() {
    let err = Server::new().rocket.launch();
}

