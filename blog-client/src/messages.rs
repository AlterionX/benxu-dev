use crate::{
    locations::{Location, M as LocationM, editor, listing, login},
    model,
    requests::PostQuery,
    shared::Authorization,
};
use boolinator::Boolinator;
use tap::*;
use serde::{Deserialize, Serialize};

pub trait AsyncM: std::future::Future<Output = M> {}
impl<T: std::future::Future<Output = M>> AsyncM for T {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostAccessMethod {
    ById(uuid::Uuid),
    ByShortName(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum M {
    // Menu is outside of seed, so we need a special message to swap it with the logged in vs the
    // not logged in version.
    ChangeMenu(Authorization),
    // Change locations
    ChangePage(Location),
    ChangePageAndUrl(Location),
    RenderPage(Location),
    // Global state
    StoreOpWithAction(
        model::StoreOperations,
        // Uses a pointer to get around the lack of default impls for references in functions
        // TODO fix when this gets resolved
        fn(*const model::Store) -> M,
    ),
    StoreOpWithMessage(model::StoreOperations, fn() -> M),
    StoreOp(model::StoreOperations),
    // Location specific
    Location(LocationM),
    // Empty message
    NoOp,
    // Chained message
    Grouped(Vec<M>),
}
impl Default for M {
    fn default() -> Self {
        Self::NoOp
    }
}
impl From<model::StoreOperations> for M {
    fn from(sop: model::StoreOperations) -> Self {
        Self::StoreOp(sop)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteMatch(Option<M>);
impl RouteMatch {
    pub fn into_inner(self) -> Option<M> {
        self.0
    }
    fn msg_from_url(url: seed::Url) -> Option<M> {
        log::info!("Routing url {:?}.", url);
        let path = url.path();
        // Verify that the first path component is "blog".
        // TODO fix routing to other pages -- this initial check of the root should route instead of return a noop.
        (path.get(0).map(String::as_str) == Some("blog"))
            .as_option()
            .tap_none(|| log::warn!("Url is missing the root path component."))?;
        let root = path.get(1)
            .map(String::as_str)
            .map(|s| if s == "" { "home" } else { s })
            .unwrap_or("home");
        let potential_id = path.get(2)
            .map(String::as_str);
        log::debug!("Proceeding to route detected root {:?} and resource {:?}.", root, potential_id);

        let loc = match (root.as_ref(), potential_id) {
            // TODO convert next two patterns into or-patterns when the feature is implemented.
            ("home", None) | ("home", Some("")) => {
                use std::convert::TryFrom;
                let search = url.search();
                let query = if search.iter().count() == 0 {
                    let q = PostQuery::try_from(url.search())
                        .tap_err(|e| log::error!("Attempting to parse url {:?} led to error: {}.", url, e))
                        .ok()?;
                    Some(q)
                } else {
                    None
                };
                Location::Listing(listing::S {
                    query,
                })
            },
            ("posts", Some(id)) => {
                let marker: model::PostMarker = id.into();
                Location::Viewer(marker.into())
            },
            ("editor", id) => Location::Editor(match id {
                None | Some("new") => editor::S::default(),
                Some(id) => {
                    let marker: model::PostMarker = id.into();
                    marker.into()
                }
            }),
            ("login", None) | ("login", Some("")) => Location::Login(login::S::default()),
            ("logout", None) | ("logout", Some("")) => Location::Logout,
            _ => Location::NotFound,
        };
        Some(M::ChangePage(loc))
    }
}
impl From<seed::Url> for RouteMatch {
    fn from(url: seed::Url) -> Self {
        Self(Self::msg_from_url(url))
    }
}
