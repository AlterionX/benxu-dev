//! Database + [`rocket`] compatibility + db queries.

use rocket::{request::FromFormValue, http::RawStr};
use rocket_contrib::database;

use chrono::{DateTime, Utc};

use blog_db::{models::*, schema};
use diesel::prelude::*;

#[database("blog")]
pub struct DB(PgConnection);

/// Logistics implementation.
impl DB {
    /// Access a reference to the connection actually used to connect to the DB. Deref gives the
    /// actual struct [`PgConnection`].
    fn conn(&self) -> &PgConnection {
        &self.0
    }
}

pub enum OrderingField {
    Date, AlphabeticalTitle
}
impl<'v> FromFormValue<'v> for OrderingField {
    type Error = &'v RawStr;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.as_str() {
            "date" => Ok(Self::Date),
            "title" => Ok(Self::AlphabeticalTitle),
            _ => Err(form_value),
        }
    }
}
pub enum SortOrdering {
    Ascending, Descending
}
impl<'v> FromFormValue<'v> for SortOrdering {
    type Error = &'v RawStr;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.as_str() {
            "asc" => Ok(Self::Descending),
            "dsc" => Ok(Self::Ascending),
            _ => Err(form_value),
        }
    }
}
/// A set of conditions for obtaining a list of posts.
pub enum PostListing {
    /// Getting posts by date.
    Date {
        /// The minimum of the time range to look at when searching.
        start: DateTime<Utc>,
        /// The maximum of the time range to look at when searching.
        stop: DateTime<Utc>,
        /// The field to order by.
        order_by: OrderingField,
        /// The direction to sort by.
        ord: SortOrdering,
        /// The maximum number of items returned.
        limit: usize,
    },
    /// Getting posts by a limit and offset.
    LimAndOffset {
        /// The offset at which to start adding to the list of posts.
        offset: usize,
        /// The number of posts, at most, to return.
        lim: usize,
        /// The field to order by.
        order_by: OrderingField,
        /// The direction to sort by.
        ord: SortOrdering,
    },
}
impl DB {
    /// Find posts based on the provided conditions.
    pub fn find_posts_with_post_listing_conditions(&self, conditions: PostListing) -> Result<Vec<posts::BasicData>, diesel::result::Error> {
        match conditions {
            PostListing::Date {
                start,
                stop,
                order_by,
                ord,
                limit,
            } => {
                let query = schema::posts::table
                    .select(posts::BasicData::COLUMNS)
                    .filter(
                        schema::posts::published_at.gt(start)
                        .and(schema::posts::published_at.lt(stop))
                    )
                    .limit(limit as i64);
                // TODO try to make this better (maybe Box?)
                match order_by {
                    OrderingField::Date => match ord {
                        SortOrdering::Ascending => query
                            .order(schema::posts::published_at.asc())
                            .load(self.conn()),
                        SortOrdering::Descending => query
                            .order(schema::posts::published_at.desc())
                            .load(self.conn()),
                    },
                    OrderingField::AlphabeticalTitle => match ord {
                        SortOrdering::Ascending => query
                            .order(schema::posts::title.asc())
                            .load(self.conn()),
                        SortOrdering::Descending => query
                            .order(schema::posts::title.desc())
                            .load(self.conn()),
                    },
                }
            },
            PostListing::LimAndOffset {
                offset,
                lim,
                order_by,
                ord,
            } => {
                let query = schema::posts::table;
                match order_by {
                    OrderingField::Date => match ord {
                        SortOrdering::Ascending => query
                            .select(posts::BasicData::COLUMNS)
                            .order(schema::posts::published_at.asc())
                            .offset(offset as i64)
                            .limit(lim as i64)
                            .load(self.conn()),
                        SortOrdering::Descending => query
                            .select(posts::BasicData::COLUMNS)
                            .order(schema::posts::published_at.desc())
                            .offset(offset as i64)
                            .limit(lim as i64)
                            .load(self.conn()),
                    },
                    OrderingField::AlphabeticalTitle => match ord {
                        SortOrdering::Ascending => query
                            .select(posts::BasicData::COLUMNS)
                            .order(schema::posts::title.asc())
                            .offset(offset as i64)
                            .limit(lim as i64)
                            .load(self.conn()),
                        SortOrdering::Descending => query
                            .select(posts::BasicData::COLUMNS)
                            .order(schema::posts::title.desc())
                            .offset(offset as i64)
                            .limit(lim as i64)
                            .load(self.conn()),
                    },
                }
            },
        }
    }

    /// Inserts the provided new post into the database. Returns the inserted post on success.
    pub fn insert_post<'a, N: Into<posts::NewWithId<'a>>>(
        &self,
        new: N,
    ) -> Result<posts::Data, diesel::result::Error> {
        diesel::insert_into(schema::posts::table)
            .values(&new.into())
            .get_result(self.conn())
    }
    /// Find the provided new post into the database. Returns the inserted post on success.
    pub fn find_post_with_id(&self, id: uuid::Uuid) -> Result<posts::Data, diesel::result::Error> {
        schema::posts::table.find(id).get_result(self.conn())
    }
    /// Given an id and a changeset, update the matching row. Returns either the number of rows
    /// updated or an error.
    #[must_use]
    pub fn update_post_with_id(
        &self,
        id: uuid::Uuid,
        update: &posts::Changed,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(update)
            .execute(self.conn())
    }
    /// Given an id, delete the matching row. Returns either the number of rows updated or an
    /// error.
    #[must_use]
    pub fn delete_post_with_id(
        &self,
        id: uuid::Uuid,
        deletion: &posts::Deletion,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(deletion)
            .execute(self.conn())
    }
    /// Given an id, publish the matching row. Returns either the number of rows updated or an
    /// error.
    pub fn publish_post_with_id(
        &self,
        id: uuid::Uuid,
        publishing: posts::Publishing,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(publishing)
            .execute(self.conn())
    }
    /// Given an id, archive the matching row. Returns either the number of rows updated or an
    /// error.
    pub fn archive_post_with_id(
        &self,
        id: uuid::Uuid,
        archival: posts::Archival,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(archival)
            .execute(self.conn())
    }
}
impl DB {
    /// Locate a user given an id.
    pub fn find_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        schema::users::table.find(id).get_result(self.conn())
    }
    /// Locate a user given an user name.
    pub fn find_user_by_user_name(
        &self,
        user_name: &str,
    ) -> Result<users::Data, diesel::result::Error> {
        schema::users::table
            .filter(schema::users::user_name.eq(user_name))
            .get_result(self.conn())
    }
    /// Create a user from the provided user info.
    pub fn create_user<'a, N: Into<users::NewWithId<'a>>>(
        &self,
        new_user: N,
    ) -> Result<users::Data, diesel::result::Error> {
        diesel::insert_into(schema::users::table)
            .values(&new_user.into())
            .get_result(self.conn())
    }
    /// Delete a user given the id.
    pub fn delete_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        diesel::delete(schema::users::table.find(id)).get_result(self.conn())
    }
    /// Updates a user given the id and change set.
    pub fn update_user_by_id(
        &self,
        id: uuid::Uuid,
        update: users::Changed<'_>,
    ) -> Result<users::Data, diesel::result::Error> {
        diesel::update(schema::users::table.find(id))
            .set(update)
            .get_result(self.conn())
    }
}
impl DB {
    /// Given a user, find all matching password hashes. There should only be one.
    pub fn find_pw_hash_by_user(
        &self,
        user: &users::Data,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        credentials::pw::Data::belonging_to(user).first(self.conn())
    }
    /// Given the password's id, find it.
    pub fn find_pw_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        schema::passwords::table.find(id).get_result(self.conn())
    }
    /// Create a password hash given some information.
    pub fn create_pw_hash(
        &self,
        new_pw: credentials::pw::New,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::insert_into(schema::passwords::table)
            .values(&credentials::pw::NewWithId::from(new_pw))
            .get_result(self.conn())
    }
    /// Update a password hash given the user id and changes.
    pub fn update_pw_hash_for_user_id(
        &self,
        user_id: uuid::Uuid,
        changed_pw: credentials::pw::Changed,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::update(schema::passwords::table.filter(schema::passwords::user_id.eq(user_id)))
            .set(&changed_pw)
            .get_result(self.conn())
    }
    /// Count the number of password hashes given the user.
    pub fn count_pw_by_user(&self, user: &users::Data) -> Result<usize, diesel::result::Error> {
        credentials::pw::Data::belonging_to(user)
            .count()
            .execute(self.conn())
    }
    /// Delete the password hash given its id in the database.
    pub fn delete_pw_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::delete(schema::passwords::table.find(id)).get_result(self.conn())
    }
}
impl DB {
    /// Get permissions based on the user.
    pub fn get_user_permissions(
        &self,
        user: &users::Data,
    ) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        permissions::Data::belonging_to(user).load(self.conn())
    }
    /// Create all permissions in the [`Vec`].
    pub fn create_all_permissions<'a>(
        &'_ self,
        permissions: Vec<permissions::New<'a>>,
    ) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        let to_create: Vec<permissions::NewWithId> =
            permissions.into_iter().map(|new| new.into()).collect();
        diesel::insert_into(schema::permissions::table)
            .values(to_create)
            .get_results(self.conn())
    }
    /// Get all permissions matching the provided id. There should only be one.
    pub fn get_permission_with_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<permissions::Data, diesel::result::Error> {
        schema::permissions::table.find(id).get_result(self.conn())
    }
    /// Delete all permissions matching the provided id. There should only be one.
    pub fn delete_permission_with_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<permissions::Data, diesel::result::Error> {
        diesel::delete(schema::permissions::table.find(id)).get_result(self.conn())
    }
    /// Delete all permissions matching the provided user_id. There can (will usually be) multiple.
    pub fn delete_permissions_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        diesel::delete(schema::permissions::table.filter(schema::permissions::user_id.eq(user_id)))
            .get_results(self.conn())
    }
    /// Delete all permissions with the listed ids.
    pub fn delete_permissions_with_ids(
        &self,
        permission_ids: &[uuid::Uuid],
    ) -> Result<Vec<permissions::Data>, diesel::result::Error> {
        diesel::delete(
            schema::permissions::table.filter(schema::permissions::id.eq_any(permission_ids)),
        )
        .get_results(self.conn())
    }
}

// TODO tests?
