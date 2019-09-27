use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use tap::*;

use db_models::models::{posts, users};
use crate::{
    messages::M,
    locations::*,
    requests,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
    pub nickname: String,
}
impl Name {
    fn to_view(&self) -> seed::dom_types::Node<M> {
        p![format!("By {} {}", self.first, self.last)]
    }
}
#[derive(Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: Name,
    pub can_see_unpublished: bool,
}

#[derive(Debug, Clone)]
pub enum StoreOperations {
    LoadPost(requests::PostMarker, FetchObject<posts::DataNoMeta>),
    CachePost(requests::PostMarker, FetchObject<posts::DataNoMeta>),
    StorePostListing(requests::PostQuery, FetchObject<Vec<posts::BasicData>>),
    UpdateUser(FetchObject<users::DataNoMeta>),
}
#[derive(Debug, Default)]
pub struct Store {
    pub posts: Option<Vec<posts::BasicData>>,
    pub post: Option<posts::DataNoMeta>,
    pub user: Option<User>,
}
impl Store {
    pub fn exec(&mut self, op: StoreOperations) -> Result<Option<Location>, ()> {
        match op {
            StoreOperations::StorePostListing(q, fetched) => {
                // TODO use query data to implement cache.
                let fetched = fetched.response()
                    .map_err(|_| ())?;
                self.posts.replace(fetched.data);
                Ok(Some(Location::Listing(listing::S { query: Some(q) })))
            },
            StoreOperations::UpdateUser(fo) => {
                let fetched = fo.response()
                    .tap_err(|e| crate::log(format!("Error {:?} occurred! TODO: show an error.", e).as_str()))
                    .map_err(|_| ())?;
                let unparsed = fetched.data;
                let parsed = crate::model::User {
                    id: unparsed.id,
                    name: crate::model::Name {
                        first: unparsed.first_name.unwrap_or("unknown".to_owned()),
                        last: unparsed.last_name.unwrap_or("unknown".to_owned()),
                        nickname: "unknown".to_owned(),
                    },
                    can_see_unpublished: false,
                };
                self.user.replace(parsed);
                Ok(None)
            },
            e @ _ => unimplemented!("Attempt to load unimplemented {:?}", e),
        }
    }
}

#[derive(Default)]
pub struct Model {
    pub store: Store,
    pub loc: Location,
}
impl Model {
    pub fn to_view(&self) -> impl View<M> {
        self.loc.to_view(&self.store)
    }
}
