use axum::{http::{StatusCode, HeaderMap, HeaderValue}, response::Html};

use crate::internal_error;

pub async fn file() -> Result<(StatusCode, HeaderMap, Vec<u8>), (StatusCode, Html<String>)> {
    let path = "public/resume/resume.pdf";
    let Ok(data) = std::fs::read(path) else {
        return Err(internal_error::page().await);
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str("application/pdf").expect("media type is a valid header"));
    Ok((StatusCode::OK, headers, data))
}
