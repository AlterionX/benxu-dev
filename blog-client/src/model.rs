use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use db_models::models::posts;
use crate::{
    messages::M,
    locations::*,
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

#[derive(Default)]
pub struct Store {
    pub posts: Option<Vec<posts::BasicData>>,
    pub post: Option<posts::DataNoMeta>,
    pub user: Option<User>,
}
impl Store {
    fn update_with(&mut self, data: LocationWithData) -> Result<(), M> {
        match data {
            LocationWithData::Home(q, fetched) => {
                // TODO use query data to implement cache.
                let fetched = fetched.response()
                    .map_err(|e| M::ChangePage(Location::Home(q)))?;
                self.posts.replace(fetched.data);
                Ok(())
            },
            _ => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct Model {
    pub store: Store,
    pub loc: Location,
}
impl Model {
    pub fn update_with(&mut self, data: LocationWithData) -> Result<(), M> {
        self.store.update_with(data)
    }
    pub fn to_view(&self) -> impl View<M> {
        self.loc.to_view(&self.store)
    }
}
