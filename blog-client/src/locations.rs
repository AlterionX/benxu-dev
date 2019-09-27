use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use db_models::models::{posts, users};
use crate::{
    messages::M as GlobalM,
    model::{Model as GlobalModel, Store as GlobalS, StoreOperations as GSOp},
    requests::{PostMarker, PostQuery},
};

pub mod login;
pub mod listing;
pub mod viewer;
pub mod editor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Location {
    Login(login::S),
    Viewer(viewer::S),
    Listing(listing::S),
    Editor(editor::S),
    NotFound,
}
impl Default for Location {
    fn default() -> Self {
        Self::NotFound
    }
}
impl Location {
    pub fn find_redirect(self, s: &GlobalS) -> Result<Self, Self> {
        match self {
            Location::Login(_) => Err(self), // TODO redirect to last page if already logged in.
            _ => Err(self),
        }
    }
    pub fn fetch_req(self, gs: &GlobalS) -> Result<impl Future<Item = GlobalM, Error = GlobalM>, Self> {
        use seed::fetch::Request;
        match self {
            Location::Listing(store) => {
                const POSTS_URL: &'static str = "/api/posts";
                let query = store.query.unwrap_or_else(PostQuery::default);
                let url = format!("{}?{}", POSTS_URL, query);
                // TODO figure out caching and determing if data already loaded instead of going
                // straight to server all the time.
                if let None = gs.posts {
                } else {
                }
                Ok(Request::new(url).fetch_json(move |fo| GlobalM::StoreOp(
                    GSOp::StorePostListing(query, fo),
                    |res| res.map_or_else(|_| None, |l| l.map(GlobalM::RenderPage)),
                )))
            },
            Location::Login(_) => Err(self),
            Location::Viewer(_) => Err(self),
            Location::Editor(_) => Err(self),
            Location::NotFound => Err(self),
        }
    }
}
impl Location {
    pub fn prep_page_for_render(self, _prev: &Location, gs: &GlobalS, orders: &mut impl Orders<GlobalM, GlobalM>) {
        let loc = match self.find_redirect(&gs) {
            Ok(redirect) => {
                crate::log("Attempt to get another page.");
                orders.skip().send_msg(GlobalM::ChangePage(redirect));
                return;
            },
            Err(loc) => loc,
        };
        match loc.fetch_req(gs) {
            Ok(req) => {
                crate::log("Attempt to fetch data.");
                orders.skip().perform_cmd(req);
            },
            Err(loc) => {
                crate::log("Attempt to render page directly, since data is already present.");
                orders.skip().send_msg(GlobalM::RenderPage(loc));
            },
        }
    }
    pub fn to_view(&self, gs: &GlobalS) -> Vec<Node<GlobalM>> {
        match self {
            Location::Listing(home_store) => listing::render(home_store, gs)
                .map_message(GlobalM::Listing),
            Location::Login(login_store) => vec![
                login::render(login_store, gs)
                    .map_message(GlobalM::Login)
            ],
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

