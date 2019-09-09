use crate::schema::*;
use serde::{
    Serialize,
    Deserialize,
};

#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name="tags"]
pub struct Tag {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
}

#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name="tags"]
pub struct NewTagWithId<'a> {
    id: uuid::Uuid,
    name: &'a str,
    description: &'a str,
    created_by: uuid::Uuid,
}
impl<'a> From<NewTag<'a>> for NewTagWithId<'a> {
    fn from(new_tag: NewTag<'a>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: new_tag.name,
            description: new_tag.description,
            created_by: new_tag.created_by,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewTag<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub created_by: uuid::Uuid,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name="tags"]
pub struct UpdateTag {
    pub name: Option<String>,
    pub description: Option<String>,
}

