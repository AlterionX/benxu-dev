use rocket_contrib::database;

use diesel::prelude::*;
use blog_db::{
    models::*,
    schema,
};

#[database("blog")]
pub struct DB(diesel::PgConnection);
impl DB {
    pub fn insert_post(&self, new_post: NewPost) -> Result<Post, diesel::result::Error> {
        diesel::insert_into(schema::posts::table)
            .values(&new_post)
            .get_result(&**self)
    }
    pub fn find_post_with_id(&self, id: i32) -> Result<Post, diesel::result::Error> {
        schema::posts::table
            .find(id)
            .get_result(&**self)
    }
    pub fn update_post_with_id(&self, id: i32, update: &UpdatePost) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(update)
            .execute(&**self)
    }
    pub fn delete_post_with_id(&self, id: i32) -> Result<usize, diesel::result::Error> {
        diesel::delete(schema::posts::table.find(id))
            .execute(&**self)
    }
}

