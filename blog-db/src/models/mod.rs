//! Models used to represent specific queries in diesel.
//!
//! Some are not queries, but rather convenience

pub mod credentials;
pub mod permissions;
pub mod post_tag_junctions;
pub mod posts;
pub mod tags;
pub mod users;

// impl for utilizing Option<DateTime<_>> in db models
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

/// Format for chrono to serialize from.
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// A simple deserialization protocol for [`DateTime`].
fn datefmt<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
}

/// A simple deserialization protocol for [`Option<DateTime>`](std::option::Option).
fn option_datefmt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "datefmt")] DateTime<Utc>);

    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}
