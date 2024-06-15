use std::path::PathBuf;

use sanitise_file_name::sanitize;

use axum::{http::{HeaderMap, StatusCode, HeaderValue}, response::Html};

use crate::not_found;

type StaticFile = Result<(StatusCode, HeaderMap, Vec<u8>), (StatusCode, Html<String>)>;

macro_rules! static_file_accessor {
    ($n:ident, $mty:literal, $use_utf8:expr) => {
        pub async fn $n(axum::extract::Path(s): axum::extract::Path<String>) -> StaticFile {
            let mut p = std::path::PathBuf::try_from("public/").expect("path is always valid");
            p.push(stringify!($n));
            $crate::static_file::accessor(
                p,
                stringify!(.$n),
                s,
                $mty,
                $use_utf8,
            ).await
        }
    };
}

static_file_accessor!(css, "text/css", true);

static_file_accessor!(js, "text/javascript", true);
static_file_accessor!(wasm, "application/wasm", false);

static_file_accessor!(png, "image/png", false);
static_file_accessor!(jpg, "image/jpeg", false);
static_file_accessor!(svg, "image/svg+xml", true);

async fn accessor(
    root: PathBuf,
    required_ending: &'static str,
    unsafe_filename: String,
    media_type: &'static str,
    use_utf8: bool,
) -> StaticFile {
    trc::info!("Processing static file request {root:?}, {required_ending:?}, {unsafe_filename:?}");

    let filename = sanitize(unsafe_filename.as_str());
    if !filename.ends_with(required_ending) {
        trc::warn!("Attempted to access bad filename.");
        return Err(not_found::page().await);
    }

    let path = root.join(filename);

    let Ok(data) = std::fs::read(path.as_path()) else {
        trc::warn!("Failed to access file {:?}.", path.as_os_str());
        return Err(not_found::page().await);
    };

    let mut headers = HeaderMap::new();
    let content_type = if use_utf8 {
        HeaderValue::from_str(format!("{media_type}; charset=utf-8").as_str()).expect("media type is valid header")
    } else {
        HeaderValue::from_str(media_type).expect("media type is valid header")
    };
    headers.insert("Content-Type", content_type);
    Ok((StatusCode::OK, headers, data))
}
