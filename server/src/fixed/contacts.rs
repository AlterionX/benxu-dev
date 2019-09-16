//! Provides an endpoint and auxilliary functions for contact information.

use rocket::http::Status;

/// Returns the "contacts" page. Not yet implemented.
#[get("/contacts")]
pub fn get() -> Status {
    Status::NotImplemented
}
