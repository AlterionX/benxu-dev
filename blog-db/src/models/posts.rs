use crate::{schema::*, models::option_datefmt};
use serde::{
    Serialize,
    Deserialize,
};
use chrono::{DateTime, Utc};

#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Data {
    pub id: uuid::Uuid,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name="posts"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    title: &'a str,
    body: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: new.title,
            body: new.body,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="posts"]
pub struct Changed<'a> {
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    #[serde(default, deserialize_with = "option_datefmt")]
    pub published_at: Option<DateTime<Utc>>,
}

