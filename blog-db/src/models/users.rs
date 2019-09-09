use crate::schema::*;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name="users"]
pub struct Data {
    pub id: uuid::Uuid,
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name="users"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    user_name: &'a str,
    created_by: uuid::Uuid,
    updated_by: uuid::Uuid,
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
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
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

