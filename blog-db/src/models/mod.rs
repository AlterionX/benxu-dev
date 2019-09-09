pub mod users;
pub mod posts;
pub mod tags;
pub mod credentials;
pub mod post_tag_junctions;

// impl for utilizing Option<DateTime<_>> in db models
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

fn datefmt<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
}

fn option_datefmt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "datefmt")] DateTime<Utc>);

    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

