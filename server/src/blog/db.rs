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
    pub fn delete_post_with_id(&self, id: uuid::Uuid, deletion: &posts::Deletion) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(deletion)
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
    pub fn find_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        schema::users::table
            .find(id)
            .get_result(self.conn())
    }
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
    pub fn delete_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        diesel::delete(schema::users::table.find(id))
            .get_result(self.conn())
    }
    pub fn update_user_by_id(&self, id: uuid::Uuid, update: users::Changed<'_>) -> Result<users::Data, diesel::result::Error> {
        diesel::update(schema::users::table.find(id))
            .set(update)
            .get_result(self.conn())
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
    pub fn update_pw_hash_for_user_id(&self, user_id: uuid::Uuid, changed_pw: credentials::pw::Changed) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::update(schema::passwords::table.filter(schema::passwords::user_id.eq(user_id)))
            .set(&changed_pw)
            .get_result(self.conn())
    }
    pub fn count_pw_by_user(&self, user: &users::Data) -> Result<usize, diesel::result::Error> {
        credentials::pw::Data::belonging_to(user).count().execute(self.conn())
    }
    pub fn delete_pw_by_id(&self, id: uuid::Uuid) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::delete(schema::passwords::table.find(id))
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
        let to_create: Vec<permissions::NewWithId> = permissions.into_iter().map(|new| new.into()).collect();
        diesel::insert_into(schema::permissions::table)
            .values(to_create)
            .get_results(self.conn())
    }
    pub fn get_permission_with_id(&self, id: uuid::Uuid) -> Result<permissions::Data, diesel::result::Error> {
        schema::permissions::table
            .find(id)
            .get_result(self.conn())
    }
    pub fn delete_permission_with_id(&self, id: uuid::Uuid) -> Result<permissions::Data, diesel::result::Error> {
        diesel::delete(schema::permissions::table.find(id))
            .get_result(self.conn())
    }
    pub fn delete_permissions_by_user_id(&self, user_id: uuid::Uuid) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        diesel::delete(schema::permissions::table.filter(schema::permissions::user_id.eq(user_id)))
            .get_results(self.conn())
    }
    pub fn delete_permissions_with_ids(&self, permission_ids: &[uuid::Uuid]) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        diesel::delete(schema::permissions::table.filter(schema::permissions::id.eq_any(permission_ids)))
            .get_results(self.conn())
    }
}

#[cfg(test)]
mod test_db {
}
