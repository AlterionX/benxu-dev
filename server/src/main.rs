#![feature(
    proc_macro_hygiene,
    type_ascription,
    decl_macro,
    try_trait,
    result_map_or_else
)]

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

mod blog;
mod fixed;

mod shared_html {
    pub fn logo_markup() -> Option<page_client::data::Logo<'static>> {
        Some(page_client::data::Logo {
            src: "/public/img/branding.svg",
            href: Some("/"),
        })
    }
}

/// Algorithm utilized for hashing passwords
type PWAlgo = crypto::algo::hash::argon2::d::Algo;
type PWKeyFixture = Arc<crypto::StableKeyStore<PWAlgo>>;
/// Algorithm utilized for encrypting tokens.
type TokenAlgo = crypto::token::paseto::v2::local::Algo;
type TokenKeyStore = crypto::RotatingKeyStore<TokenAlgo>;
type TokenKeyFixture = crypto::RotatingKeyFixture<TokenAlgo>;

/// A struct to ensure correct initialization of the server.
struct Server {
    /// Path to configuration file as per dotenv.
    _env_path: std::path::PathBuf,
    /// SodiumOxide crypto library initialization -- used as a reminder.
    _sodiumoxide_init: (),
    /// Key store + key rotation for the PASETO v2 local tokens used for authz.
    _paseto_key: crypto::KeyRotator<TokenAlgo>,
    /// Key store for passwords secret keys.
    _local_loaded_key: Arc<crypto::StableKeyStore<PWAlgo>>,
    /// The Rocket instance managing all handlers and data routing.
    rocket: Option<rocket::Rocket>,
}
impl Server {
    /// Default path for the password secret.
    const PW_SECRET_KEY_DEFAULT_PATH: &'static str = "./.pw_secret";
    /// Name for environment variable holding path to password secret key.
    const PW_SECRET_KEY_ENV_VAR_NAME: &'static str = "BENXU_DEV_PW_SECRET";

    /// Routing path root for static pages from the [`fixed`](crate::fixed) module.
    const STATIC_ROOT: &'static str = "/";
    /// Routing path root for static resources.
    const PUBLIC_ROOT: &'static str = "/public";
    /// Filesystem path root for static resources.
    const PUBLIC_DIRECTORY: &'static str = "./public";
    /// Routing path root for blog pages/endpoints from the [`blog`](crate::blog) module.
    const BLOG_API_ROOT: &'static str = "/api";
    /// Routing path root for blog pages/endpoints from the [`blog`](crate::blog) module.
    const BLOG_SPA_ROOT: &'static str = "/blog";

    /// Seeks out and returns the path to the secret key for password hashing, returning an error
    /// if it could not find the file and defaulting to
    /// [`PW_SECRET_KEY_DEFAULT_PATH`](crate::Server::PW_SECRET_KEY_DEFAULT_PATH) if the
    /// environment variable
    /// [`PW_SECRET_KEY_ENV_VAR_NAME`](crate::Server::PW_SECRET_KEY_ENV_VAR_NAME) for that path
    /// does not exist.
    fn pw_secret() -> Result<std::path::PathBuf, std::io::Error> {
        let env_var_res = std::env::var(Self::PW_SECRET_KEY_ENV_VAR_NAME);
        let pw_file = env_var_res
            .as_ref()
            .map(|s| s.as_ref())
            .tap_err(|e| {
                match e {
                    std::env::VarError::NotPresent => warn!(
                        "The environment variable pointing to the secret key for password hashing is not present!",
                    ),
                    std::env::VarError::NotUnicode(var) => warn!(
                        "The environment variable is not unicode/ascii! Found {:?} instead.",
                        var,
                    ),
                };
                warn!("Defaulting secret key path to \"{}\".", Self::PW_SECRET_KEY_DEFAULT_PATH)
            })
            .unwrap_or(Self::PW_SECRET_KEY_DEFAULT_PATH);
        Path::new(pw_file)
            .canonicalize()
            .tap_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    error!(
                        "Could not find the secret key file \"{}\" for password hashing!",
                        pw_file
                    );
                }
                _ => error!(
                    "Unknown error encountered when attempting to file \"{}\"\n{:?}",
                    pw_file, e
                ),
            })
    }
    /// Initializes the key rotation system for the token's secret key.
    fn init_token_key() -> crypto::KeyRotator<TokenAlgo> {
        crypto::KeyRotator::init(TokenAlgo {}, None)
    }
    /// Initializes the key store for the password's hashing secret key.
    fn init_pw_secret<S: AsRef<Path>>(secret_path: &S) -> crypto::StableKeyStore<PWAlgo> {
        use std::{fs::File, io::Read};
        info!("Loading secret from disk...");
        let secret_path = secret_path.as_ref();
        let secret = File::open(secret_path)
            .and_then(|mut f| {
                info!("Loading file metadata...");
                let meta = f.metadata()?;
                if meta.len() == 0 {
                    warn!("No secret key provided!");
                    Ok(vec![])
                } else {
                    info!("Reading file...");
                    let to_read = if meta.len() > 32 {
                        warn!("Secret key far larger than expected! Truncating the secret to 2^32 - 1 bytes.");
                        32u64
                    } else if meta.len() < 16 {
                        warn!("Secret key far smaller than suggested 32 bytes! Please consider lengthening the secret.");
                        meta.len()
                    } else {
                        meta.len()
                    } as usize;
                    let mut read_bytes = vec![0; to_read];
                    f.read_exact(read_bytes.as_mut_slice())?;
                    info!("File read.");
                    Ok(read_bytes)
                }
            })
            .tap_err(|e| {
                use std::io::ErrorKind;
                match e.kind() {
                    // This should be taken care of already when finding the path, but jic.
                    ErrorKind::NotFound => error!(
                        "Could not find file at `{}`.",
                        secret_path.display(),
                    ),
                    ErrorKind::PermissionDenied => error!(
                        "Lacking valid permissions for accessing file at `{}`.",
                        secret_path.display(),
                    ),
                    _ => error!(
                        "Could not find secret key at path `{}`! Caused by: {:?}",
                        secret_path.display(),
                        e,
                    ),
                };
            })
            .expect("Located secret key file.");
        use crypto::algo::Algo as A;
        crypto::key_rotation::StableKeyStore::new(
            PWAlgo::new(None),
            <PWAlgo as A>::Key::new(secret),
        )
    }
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
            let path_str = Self::PUBLIC_DIRECTORY;
            let path = Path::new(Self::PUBLIC_DIRECTORY)
                .canonicalize()
                .tap(|r| match r {
                    Ok(p) => info!("Serving static files from `{}`.", p.display()),
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            error!("Could not find path `{}` in file system", path_str)
                        }
                        _ => error!("Unhandled IO error for path `{}`:\n{:?}", path_str, e),
                    },
                })
                .expect("a proper path.");
            info!("Crypto crate initialized.");
            path
        };
        let pw_secret_path = {
            info!("Locating password hashing secret...");
            let path = Self::pw_secret().expect("A proper path to a proper file.");
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
            let rotator = Self::init_token_key();
            info!("Token cryptographic key rotation initialized.");
            rotator
        };
        let local_loaded_key = {
            info!("Initializing password secret key...");
            let static_key_store = Arc::new(Self::init_pw_secret(&pw_secret_path));
            info!("Password secret key initialized.");
            static_key_store
        };
        // Initializing rocket and attaching all the things.
        let rocket = {
            info!("Prepping Rocket...");
            let rocket = rocket::ignite()
                .mount(Self::STATIC_ROOT, fixed::routes())
                .mount(Self::PUBLIC_ROOT, StaticFiles::from(public_path))
                .attach(blog::DB::fairing())
                .manage(Arc::clone(&local_loaded_key))
                .manage(paseto_key.get_key_fixture())
                .mount(Self::BLOG_API_ROOT, blog::api_routes())
                .mount(Self::BLOG_SPA_ROOT, blog::spa_routes());
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
