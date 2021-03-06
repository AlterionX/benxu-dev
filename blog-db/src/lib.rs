#![feature(type_ascription)]

//! A collection of types and migrations for use with diesel and postgresql specifically for my
//! website.

pub mod models;

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;
/// Auto generated by diesel. Reflects the database schema after applying all migrations in the
/// `migrations` folder.
#[cfg(feature = "diesel")]
pub mod schema;

#[cfg(feature = "client")]
pub use models::{capabilities, credentials, post_tag_junctions, posts, tags, users};

#[cfg(feature = "server")]
pub mod query;
#[cfg(feature = "server")]
pub mod rocket;
