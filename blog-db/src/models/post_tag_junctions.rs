use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(super::posts::Post)]
#[belongs_to(super::tags::Tag)]
pub struct PostTagJunction {
    #[primary_key(nonstandard)]
    pub id: (uuid::Uuid, uuid::Uuid),
    pub post_id: uuid::Uuid,
    pub tag_id: uuid::Uuid,
    pub created_by: uuid::Uuid,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "post_tag_junctions"]
pub struct NewPostTagJunction {
    pub post_id: uuid::Uuid,
    pub tag_id: uuid::Uuid,
    pub created_by: uuid::Uuid,
}
