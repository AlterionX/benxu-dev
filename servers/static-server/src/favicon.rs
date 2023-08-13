use axum::{http::{StatusCode, HeaderMap, HeaderValue}, response::Html};

use crate::internal_error;

pub async fn ico() -> Result<(StatusCode, HeaderMap, Vec<u8>), (StatusCode, Html<String>)> {
    let path = "public/ico/favicon.ico";
    let Ok(data) = std::fs::read(path) else {
        return Err(internal_error::page().await);
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str("image/vnd.microsoft.icon").expect("media type is a valid header"));
    Ok((StatusCode::OK, headers, data))
}

pub async fn svg() -> Result<(StatusCode, HeaderMap, Vec<u8>), (StatusCode, Html<String>)> {
    let path = "public/svg/branding.svg";
    let Ok(data) = std::fs::read(path) else {
        return Err(internal_error::page().await);
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str("image/svg+xml; charset=utf-8").expect("media type is a valid header"));
    Ok((StatusCode::OK, headers, data))
}
