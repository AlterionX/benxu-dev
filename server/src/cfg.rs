use crypto;
use log::*;
use std::{
    path::PathBuf,
    sync::Arc,
};
use structopt::StructOpt;
use tap::*;

/// Algorithm utilized for hashing passwords
pub type PWAlgo = crypto::algo::hash::argon2::d::Algo;
pub type PWKeyFixture = Arc<crypto::StableKeyStore<PWAlgo>>;
/// Algorithm utilized for encrypting tokens.
pub type TokenAlgo = <crypto::token::paseto::V2Local as crypto::token::paseto::Protocol>::CoreAlgo;
pub type TokenKeyStore = crypto::RotatingKeyStore<TokenAlgo>;
pub type TokenKeyFixture = crypto::RotatingKeyFixture<TokenAlgo>;

/// Default path for the password secret.
pub const PW_SECRET_KEY_DEFAULT_PATH: &'static str = "./.pw_secret";
/// Name for environment variable holding path to password secret key.
pub const PW_SECRET_KEY_ENV_VAR_NAME: &'static str = "BENXU_DEV_PW_SECRET";

/// Routing path root for static pages from the [`fixed`](crate::fixed) module.
pub const STATIC_ROOT: &'static str = "/";
/// Routing path root for static resources.
pub const PUBLIC_ROOT: &'static str = "/public";
/// Filesystem path root for static resources.
pub const PUBLIC_DIRECTORY: &'static str = "./public";
/// Routing path root for blog pages/endpoints from the [`blog`](crate::blog) module.
pub const BLOG_API_ROOT: &'static str = "/api";
/// Routing path root for blog pages/endpoints from the [`blog`](crate::blog) module.
pub const BLOG_SPA_ROOT: &'static str = "/blog";

#[derive(Debug, StructOpt)]
#[structopt(name = "benxu-server", about = "Server for benxu.dev")]
pub struct Opt {
    #[structopt(long)]
    pub ignore_dotenv: bool,
    #[structopt(
        long,
        default_value = PW_SECRET_KEY_DEFAULT_PATH,
        env = PW_SECRET_KEY_ENV_VAR_NAME
    )]
    pub pw_secret_path: PathBuf,
    #[structopt(
        long,
        default_value = PUBLIC_ROOT
    )]
    pub public_root_route: String,
    #[structopt(
        long,
        default_value = PUBLIC_DIRECTORY,
    )]
    pub public_root_dir: PathBuf,
    #[structopt(
        long,
        default_value = STATIC_ROOT,
    )]
    pub fixed_root_route: String,
    #[structopt(
        long,
        default_value = BLOG_API_ROOT,
    )]
    pub blog_api_root_route: String,
    #[structopt(
        long,
        default_value = BLOG_SPA_ROOT,
    )]
    pub blog_spa_root_route: String,
}

impl Opt {
    pub fn load() -> Opt {
        let opt = Opt::from_args();
        log::trace!("App loaded initially with following opt: {:?}", opt);
        // We need to reload again if we were not supposed to ignore the dotenv file.
        let opt = if opt.ignore_dotenv {
            opt
        } else {
            log::trace!("Loading dotenv file.");
            match dotenv::dotenv() {
                Ok(p) => log::info!("Dotenv file loaded from `{:?}`", p),
                Err(e) => log::warn!("Could not load dotenv file due to: {:?}.", e),
            };
            log::trace!("Reloading Opt from args after attempting to load dotenv.");
            Opt::from_args()
        };
        log::debug!("App loaded with following opt: {:?}", opt);
        log::info!("Configuration fully loaded.");
        opt
    }
}

/// Initializes the key rotation system for the token's secret key.
pub fn token_key() -> crypto::KeyRotator<TokenAlgo> {
    crypto::KeyRotator::init(TokenAlgo {}, None)
}

/// Initializes the key store for the password's hashing secret key.
pub fn pw_secret(opt: &Opt) -> crypto::StableKeyStore<PWAlgo> {
    use std::{fs::File, io::Read};
    log::debug!("Locating password hashing secret...");
    let secret_path = opt.pw_secret_path
        .canonicalize()
        .tap_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                error!(
                    "Could not find the secret key file `{}` for password hashing!",
                    opt.pw_secret_path.display()
                );
            }
            _ => error!(
                "Unknown error encountered when attempting to file `{}`\n{:?}",
                opt.pw_secret_path.display(), e
            ),
        })
        .expect("Password secret to be present.");
    log::info!("Password secret located.");
    log::info!("Loading secret from file {}...", secret_path.display());
    let secret = File::open(secret_path)
        .and_then(|mut f| {
            info!("Loading file metadata...");
            let meta = f.metadata()?;
            if meta.len() == 0 {
                log::warn!("No secret key provided!");
                Ok(vec![])
            } else {
                info!("Reading file...");
                let to_read = if meta.len() > 32 {
                    log::warn!("Secret key larger than expected! Truncating the secret to 32 bytes. (Check if limit is actully 2^32 - 1 bytes.)");
                    32u64
                } else if meta.len() < 16 {
                    log::warn!("Secret key far smaller than suggested 32 bytes! Please consider lengthening the secret.");
                    meta.len()
                } else {
                    meta.len()
                } as usize;
                let mut read_bytes = vec![0; to_read];
                f.read_exact(read_bytes.as_mut_slice())?;
                log::info!("File read.");
                Ok(read_bytes)
            }
        })
        .tap_err(|e| {
            use std::io::ErrorKind;
            match e.kind() {
                // This should be taken care of already when finding the path, but jic.
                ErrorKind::NotFound => log::error!(
                    "Could not find file at `{}`.",
                    opt.pw_secret_path.display()
                ),
                ErrorKind::PermissionDenied => log::error!(
                    "Lacking valid permissions for accessing file at `{}`.",
                    opt.pw_secret_path.display()
                ),
                _ => log::error!(
                    "Could not find secret key at path `{}`! Caused by: {:?}",
                    opt.pw_secret_path.display(),
                    e,
                ),
            };
        })
        .expect("Located secret key file.");
    use crypto::algo::Algo as A;
    crypto::key_rotation::StableKeyStore::new(PWAlgo::new(None), <PWAlgo as A>::Key::new(secret))
}