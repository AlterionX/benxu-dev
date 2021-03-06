//! Provides an endpoint and auxilliary functions for related links (Github, Facebook, etc).

use rocket::http::Status;

/// Returns the "links" page. Not yet implemented.
#[get("/links")]
pub fn get() -> Status {
    Status::NotImplemented
}

/// Functions generating a summary of my online presence. Not yet implemented.
pub mod htmlgen {}
