use crate::{schema::*, models};
use serde::{
    Serialize,
    Deserialize,
};
use chrono::{DateTime, Utc};

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize)]
#[belongs_to(parent = "models::users::Data", foreign_key = "user_id")]
#[table_name="permissions"]
pub struct Data {
    pub id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<uuid::Uuid>,
    pub user_id: uuid::Uuid,
    pub permission: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name="permissions"]
pub struct NewWithId<'a> {
    id: uuid::Uuid,
    created_by: uuid::Uuid,
    user_id: uuid::Uuid,
    permission: &'a str,
}
impl<'a> From<New<'a>> for NewWithId<'a> {
    fn from(new: New<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created_by: new.created_by,
            user_id: new.user_id,
            permission: new.permission,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct New<'a> {
    pub created_by: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub permission: &'a str,
}

