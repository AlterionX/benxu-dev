use rocket_contrib::database;

use diesel::prelude::*;
use blog_db::{
    models::*,
    schema,
};

#[database("blog")]
pub struct DB(diesel::PgConnection);
impl DB {
    fn conn(&self) -> &PgConnection {
        &**self
    }
}
impl DB {
    pub fn insert_post(&self, new_post: posts::New) -> Result<posts::Data, diesel::result::Error> {
        diesel::insert_into(schema::posts::table)
            .values(&posts::NewWithId::from(new_post))
            .get_result(self.conn())
    }
    pub fn find_post_with_id(&self, id: uuid::Uuid) -> Result<posts::Data, diesel::result::Error> {
        schema::posts::table
            .find(id)
            .get_result(self.conn())
    }
    pub fn update_post_with_id(&self, id: uuid::Uuid, update: &posts::Changed) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(update)
            .execute(self.conn())
    }
    pub fn delete_post_with_id(&self, id: uuid::Uuid) -> Result<usize, diesel::result::Error> {
        diesel::delete(schema::posts::table.find(id))
            .execute(self.conn())
    }
    pub fn publish_post_with_id(&self, id: uuid::Uuid, publishing: posts::Publishing) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(publishing)
            .execute(self.conn())
    }
    pub fn archive_post_with_id(&self, id: uuid::Uuid, archival: posts::Archival) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(archival)
            .execute(self.conn())
    }
}
impl DB {
    pub fn find_user_by_user_name(&self, user_name: &str) -> Result<users::Data, diesel::result::Error> {
        schema::users::table
            .filter(schema::users::user_name.eq(user_name))
            .get_result(self.conn())
    }
    pub fn create_user(&self, new_user: users::New) -> Result<users::Data, diesel::result::Error> {
        diesel::insert_into(schema::users::table)
            .values(&users::NewWithId::from(new_user))
            .get_result(self.conn())
    }
    pub fn delete_user_by_id(id: uuid::Uuid) {
    }
}
impl DB {
    pub fn find_pw_hash_by_user(&self, user: &users::Data) -> Result<credentials::pw::Data, diesel::result::Error> {
        credentials::pw::Data::belonging_to(user).first(self.conn())
    }
    pub fn create_pw_hash(&self, new_pw: credentials::pw::New) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::insert_into(schema::passwords::table)
            .values(&credentials::pw::NewWithId::from(new_pw))
            .get_result(self.conn())
    }
}
impl DB {
    pub fn create_permission<'a>(&'_ self, new_permission: permissions::New<'a>) -> Result<permissions::Data, diesel::result::Error> {
        diesel::insert_into(schema::permissions::table)
            .values(&permissions::NewWithId::from(new_permission))
            .get_result(self.conn())
    }
    pub fn get_user_permissions(&self, user: &users::Data) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        permissions::Data::belonging_to(user).load(self.conn())
    }
    pub fn create_all_permissions<'a>(&'_ self, permissions: Vec<permissions::New<'a>>) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        let mut created_permissions = Vec::with_capacity(permissions.len());
        for target_permission in permissions.into_iter() {
            created_permissions.push(self.create_permission(target_permission)?);
        }
        Ok(created_permissions)
    }
}

