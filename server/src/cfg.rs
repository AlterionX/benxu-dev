use crypto;
use log::*;
use std::{path::Path, sync::Arc};
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

/// Seeks out and returns the path to the secret key for password hashing, returning an error
/// if it could not find the file and defaulting to
/// [`PW_SECRET_KEY_DEFAULT_PATH`](crate::Server::PW_SECRET_KEY_DEFAULT_PATH) if the
/// environment variable
/// [`PW_SECRET_KEY_ENV_VAR_NAME`](crate::Server::PW_SECRET_KEY_ENV_VAR_NAME) for that path
/// does not exist.
pub fn pw_secret() -> Result<std::path::PathBuf, std::io::Error> {
    let env_var_res = std::env::var(PW_SECRET_KEY_ENV_VAR_NAME);
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
            warn!("Defaulting secret key path to \"{}\".", PW_SECRET_KEY_DEFAULT_PATH)
        })
        .unwrap_or(PW_SECRET_KEY_DEFAULT_PATH);
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
pub fn init_token_key() -> crypto::KeyRotator<TokenAlgo> {
    crypto::KeyRotator::init(TokenAlgo {}, None)
}
/// Initializes the key store for the password's hashing secret key.
pub fn init_pw_secret<S: AsRef<Path>>(secret_path: &S) -> crypto::StableKeyStore<PWAlgo> {
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
    crypto::key_rotation::StableKeyStore::new(PWAlgo::new(None), <PWAlgo as A>::Key::new(secret))
}
