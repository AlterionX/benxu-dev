use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use db_models::models::posts;
use crate::{
    messages::M as GlobalM,
    model::{Model as GlobalModel, Store as GlobalStore},
    requests::{PostMarker, PostQuery},
};

pub mod login;
pub mod home;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Location {
    Login(login::Store),
    Viewer(PostMarker),
    Home(home::Store),
    Editor(PostMarker),
    NotFound,
}
impl Default for Location {
    fn default() -> Self {
        Self::NotFound
    }
}
impl Location {
    pub fn find_redirect(self, m: &GlobalModel) -> Result<Self, Self> {
        match self {
            Location::Login(_) => Err(self), // TODO redirect to last page if already logged in.
            _ => Err(self),
        }
    }
    pub fn required_data_fetch(self, m: &GlobalModel) -> Result<impl Future<Item = GlobalM, Error = GlobalM>, Self> {
        use seed::fetch::Request;
        match self {
            Location::Home(store) => {
                let query = store.query.unwrap_or_else(PostQuery::default);
                if let None = m.store.posts {
                    const POSTS_URL: &'static str = "/api/posts";
                    let url = format!("{}?{}", POSTS_URL, query);
                    let boxed_fetch_map: Box<dyn FnOnce(_) -> _> = Box::new(
                        move |fo| GlobalM::DataFetched(LocationWithData::Home(home::Store { query: Some(query) }, fo))
                    );
                    Ok(Request::new(url).fetch_json(boxed_fetch_map))
                } else {
                    // TODO figure out caching and determing if data already loaded
                    // Err(Location::Home(query))
                    const POSTS_URL: &'static str = "/api/posts";
                    let url = format!("{}?{}", POSTS_URL, query);
                    let boxed_fetch_map: Box<dyn FnOnce(_) -> _> = Box::new(
                        move |fo| GlobalM::DataFetched(LocationWithData::Home(home::Store { query: Some(query) }, fo))
                    );
                    Ok(Request::new(url).fetch_json(boxed_fetch_map))
                }
            },
            Location::Login(_) => Err(self),
            Location::Viewer(_) => Err(self),
            Location::Editor(_) => Err(self),
            Location::NotFound => Err(self),
        }
    }
}
impl Location {
    fn spinner() -> seed::dom_types::Node<GlobalM> {
        p!["Loading..."]
    }
    pub fn to_view(&self, gs: &GlobalStore) -> Vec<seed::dom_types::Node<GlobalM>> {
        match self {
            Location::Home(home_store) => home::render(home_store, gs),
            Location::Login(_) => {
                vec![
                    form![
                        p!["Please enter your username"],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "username";
                                At::AutoFocus => true;
                                At::Type => "text";
                            },
                            input_ev(Ev::Input, |text| GlobalM::Login(login::M::UserName(text))),
                        ],
                        br![],
                        p!["Please enter your password"],
                        input![
                            attrs! {
                                At::Class => "single-line-text-entry";
                                At::Placeholder => "password";
                                At::AutoFocus => true;
                                At::Type => "password";
                            },
                            input_ev(Ev::Input, |text| GlobalM::Login(login::M::Password(text))),
                        ],
                        br![],
                        input![
                            attrs! { At::Type => "submit" },
                            "Login",
                        ],
                        raw_ev(Ev::Submit, |e| {
                            e.prevent_default();
                            // TODO actually submit
                            GlobalM::Login(login::M::FormSubmission)
                        }),
                    ],
                ]
            },
            Location::Viewer(pm) => {
                vec![]
            },
            Location::Editor(pm) => {
                vec![]
            },
            Location::NotFound => {
                vec![]
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum LocationWithData {
    Viewer(PostMarker, FetchObject<posts::DataNoMeta>),
    Home(home::Store, FetchObject<Vec<posts::BasicData>>),
    Editor(PostMarker, FetchObject<posts::DataNoMeta>),
}
impl LocationWithData {
    pub fn to_loc(&self) -> Location {
        match self {
            Self::Viewer(pm, _) => Location::Viewer(pm.clone()),
            Self::Editor(pm, _) => Location::Editor(pm.clone()),
            Self::Home(pq, _) => Location::Home(pq.clone()),
        }
    }
}

