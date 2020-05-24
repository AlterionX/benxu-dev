//! Database + [`rocket`] compatibility + db queries.

use chrono::{DateTime, Utc};

use diesel::prelude::*;
use rocket::{http::RawStr, request::FromFormValue};

use crate::{models::*, schema};

#[derive(Debug)]
pub enum OrderingField {
    Date,
    AlphabeticalTitle,
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
#[derive(Debug)]
pub enum SortOrdering {
    Ascending,
    Descending,
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
#[derive(Debug)]
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

pub trait DBConn {
    fn conn(&self) -> &PgConnection;
}

pub trait PostQuery: DBConn {
    /// Find posts based on the provided conditions.
    fn find_posts_with_post_listing_conditions(
        &self,
        conditions: PostListing,
        show_unpublished: bool,
    ) -> Result<Vec<posts::BasicData>, diesel::result::Error> {
        log::debug!("Attempting to find posts with {:?} query.", conditions);
        let query = schema::posts::table;
        let query = if show_unpublished {
            query.filter(schema::posts::published_at.is_null()).into_boxed()
        } else {
            query.into_boxed()
        };
        match conditions {
            PostListing::Date {
                start,
                stop,
                order_by,
                ord,
                limit,
            } => {
                let query = query
                    .select(posts::BasicData::COLUMNS)
                    .filter(
                        schema::posts::published_at
                            .gt(start)
                            .and(schema::posts::published_at.lt(stop)),
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
                        SortOrdering::Ascending => {
                            query.order(schema::posts::title.asc()).load(self.conn())
                        }
                        SortOrdering::Descending => {
                            query.order(schema::posts::title.desc()).load(self.conn())
                        }
                    },
                }
            }
            PostListing::LimAndOffset {
                offset,
                lim,
                order_by,
                ord,
            } => {
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
            }
        }
    }

    /// Inserts the provided new post into the database. Returns the inserted post on success.
    fn insert_post<'a, N: Into<posts::NewWithId<'a>>>(
        &self,
        new: N,
    ) -> Result<posts::Data, diesel::result::Error> {
        diesel::insert_into(schema::posts::table)
            .values(&new.into())
            .get_result(self.conn())
    }
    /// Find the provided new post into the database. Returns the inserted post on success.
    fn find_post_with_id(&self, id: uuid::Uuid) -> Result<posts::Data, diesel::result::Error> {
        schema::posts::table.find(id).get_result(self.conn())
    }
    /// Given an id and a changeset, update the matching row. Returns either the number of rows
    /// updated or an error.
    #[must_use]
    fn update_post_with_id(
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
    fn delete_post_with_id(
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
    fn publish_post_with_id(
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
    fn archive_post_with_id(
        &self,
        id: uuid::Uuid,
        archival: posts::Archival,
    ) -> Result<usize, diesel::result::Error> {
        diesel::update(schema::posts::table.find(id))
            .set(archival)
            .execute(self.conn())
    }
}
impl<T: DBConn> PostQuery for T {}

pub trait UserQuery: DBConn {
    /// Locate a user given an id.
    fn find_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        schema::users::table.find(id).get_result(self.conn())
    }
    /// Locate a user given an user name.
    fn find_user_by_user_name(
        &self,
        user_name: &str,
    ) -> Result<users::Data, diesel::result::Error> {
        use log::*;
        trace!("Searching for {:?}.", user_name);
        let query = schema::users::table.filter(schema::users::user_name.eq(user_name));
        trace!(
            "Query constructed: {}. Now running...",
            diesel::debug_query::<diesel::pg::Pg, _>(&query)
        );
        query.first(self.conn())
    }
    /// Create a user from the provided user info.
    fn create_user<'a, N: Into<users::NewWithId<'a>>>(
        &self,
        new_user: N,
    ) -> Result<users::Data, diesel::result::Error> {
        diesel::insert_into(schema::users::table)
            .values(&new_user.into())
            .get_result(self.conn())
    }
    /// Delete a user given the id.
    fn delete_user_by_id(&self, id: uuid::Uuid) -> Result<users::Data, diesel::result::Error> {
        diesel::delete(schema::users::table.find(id)).get_result(self.conn())
    }
    /// Updates a user given the id and change set.
    fn update_user_by_id(
        &self,
        id: uuid::Uuid,
        update: users::Changed<'_>,
    ) -> Result<users::Data, diesel::result::Error> {
        diesel::update(schema::users::table.find(id))
            .set(update)
            .get_result(self.conn())
    }
}
impl<T: DBConn> UserQuery for T {}

pub trait PWQuery: DBConn {
    /// Given a user, find all matching password hashes. There should only be one.
    fn find_pw_hash_by_user(
        &self,
        user: &users::Data,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        let query = credentials::pw::Data::belonging_to(user);
        log::trace!(
            "Query constructed: {}. Now running...",
            diesel::debug_query::<diesel::pg::Pg, _>(&query)
        );
        query.first(self.conn())
    }
    /// Given the password's id, find it.
    fn find_pw_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        schema::passwords::table.find(id).get_result(self.conn())
    }
    /// Create a password hash given some information.
    fn create_pw_hash(
        &self,
        new_pw: credentials::pw::New,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        use log::*;
        let record = credentials::pw::NewWithId::from(new_pw);
        let query = diesel::insert_into(schema::passwords::table).values(&record);
        debug!(
            "Running query to save password: {:?}",
            diesel::debug_query(&query)
        );
        query.get_result(self.conn())
    }
    /// Update a password hash given the user id and changes.
    fn update_pw_hash_for_user_id(
        &self,
        user_id: uuid::Uuid,
        changed_pw: credentials::pw::Changed,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::update(schema::passwords::table.filter(schema::passwords::user_id.eq(user_id)))
            .set(&changed_pw)
            .get_result(self.conn())
    }
    /// Count the number of password hashes given the user.
    fn count_pw_by_user(&self, user: &users::Data) -> Result<i64, diesel::result::Error> {
        credentials::pw::Data::belonging_to(user)
            .count()
            .get_result(self.conn())
    }
    /// Delete the password hash given its id in the database.
    fn delete_pw_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<credentials::pw::Data, diesel::result::Error> {
        diesel::delete(schema::passwords::table.find(id)).get_result(self.conn())
    }
}
impl<T: DBConn> PWQuery for T {}

pub trait CapabilityQuery: DBConn {
    /// Get capabilities based on the user.
    fn get_user_capabilities(
        &self,
        user: &users::Data,
    ) -> Result<Vec<capabilities::Data>, diesel::result::Error> {
        capabilities::Data::belonging_to(user).load(self.conn())
    }
    /// Create all capabilities in the [`Vec`].
    fn create_all_capabilities<'a>(
        &'_ self,
        capabilities: Vec<capabilities::New<'a>>,
    ) -> Result<Vec<capabilities::Data>, diesel::result::Error> {
        let to_create: Vec<capabilities::NewWithId> =
            capabilities.into_iter().map(|new| new.into()).collect();
        diesel::insert_into(schema::capabilities::table)
            .values(to_create)
            .get_results(self.conn())
    }
    /// Get all capabilities matching the provided id. There should only be one.
    fn get_capability_with_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<capabilities::Data, diesel::result::Error> {
        schema::capabilities::table.find(id).get_result(self.conn())
    }
    /// Delete all capabilities matching the provided id. There should only be one.
    fn delete_capability_with_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<capabilities::Data, diesel::result::Error> {
        diesel::delete(schema::capabilities::table.find(id)).get_result(self.conn())
    }
    /// Delete all capabilities matching the provided user_id. There can (will usually be) multiple.
    fn delete_capabilities_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<capabilities::Data>, diesel::result::Error> {
        diesel::delete(
            schema::capabilities::table.filter(schema::capabilities::user_id.eq(user_id)),
        )
        .get_results(self.conn())
    }
    /// Delete all capabilities with the listed ids.
    fn delete_capabilities_with_ids(
        &self,
        capability_ids: &[uuid::Uuid],
    ) -> Result<Vec<capabilities::Data>, diesel::result::Error> {
        diesel::delete(
            schema::capabilities::table.filter(schema::capabilities::id.eq_any(capability_ids)),
        )
        .get_results(self.conn())
    }
}
impl<T: DBConn> CapabilityQuery for T {}

// TODO tests?
