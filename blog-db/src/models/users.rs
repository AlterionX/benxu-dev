use crate::schema::*;
use serde::{
    Serialize,
    Deserialize,
};
use chrono::{DateTime, Utc};

#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name="users"]
pub struct Data {
    pub id: uuid::Uuid,
    pub user_name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<uuid::Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<uuid::Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name="users"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    user_name: &'a str,
    created_by: Option<uuid::Uuid>,
    updated_by: Option<uuid::Uuid>,
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            user_name: new.user_name,
            created_by: new.created_by,
            updated_by: new.updated_by,
            first_name: new.first_name,
            last_name: new.last_name,
            email: new.email,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    pub user_name: &'a str,
    pub created_by: Option<uuid::Uuid>,
    pub updated_by: Option<uuid::Uuid>,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
}
impl<'a> From<(&'a NewNoMeta, Option<uuid::Uuid>)> for New<'a> {
    fn from((source, user): (&'a NewNoMeta, Option<uuid::Uuid>)) -> Self {
        Self {
            user_name: &source.user_name,
            created_by: user,
            updated_by: user,
            first_name: &source.first_name,
            last_name: &source.last_name,
            email: &source.email,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewNoMeta {
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="users"]
pub struct Changed<'a> {
    pub user_name: Option<&'a str>,
    pub updated_by: Option<uuid::Uuid>,
    pub first_name: Option<&'a str>,
    pub last_name: Option<&'a str>,
    pub email: Option<&'a str>,
}
