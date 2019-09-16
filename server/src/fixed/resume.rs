//! Provides an endpoint and auxilliary functions for resume data.

use rocket::http::Status;

/// Returns the "resume" page. Not yet implemented.
#[get("/resume")]
pub fn get() -> Status {
    Status::NotImplemented
}
