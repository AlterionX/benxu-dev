use crate::{locations::*, model, requests::PostQuery, shared::Authorization};
use boolinator::Boolinator;
use serde::{Deserialize, Serialize};

pub trait AsyncM: std::future::Future<Output = Result<M, M>> {}
impl<T: std::future::Future<Output = Result<M, M>>> AsyncM for T {}

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
    // Globabl state
    StoreOpWithAction(
        model::StoreOperations,
        // Uses a pointer to get around the lack of default impls for references in functions
        // TODO fix when this gets resolved
        fn(*const model::Store, model::StoreOpResult) -> Option<M>,
    ),
    StoreOp(model::StoreOperations),
    // Location specific
    Login(login::M),
    Editor(editor::M),
    Viewer(viewer::M),
    Listing(listing::M),
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
    fn msg_from_url(mut url: seed::Url) -> Option<M> {
        let potential_id = (url.path.len() >= 3).as_some_from(|| url.path.remove(2));
        let potential_resource = (url.path.len() >= 2).as_some_from(|| url.path.remove(1));
        let root = (!url.path.is_empty()).as_some_from(|| url.path.remove(0));
        let root_str: &str = root.as_ref()?.as_str();
        let root = (root_str == "blog")
            .as_some_from(|| potential_resource)?
            .map(|s| if s == "" { "home".to_owned() } else { s })
            .unwrap_or_else(|| "home".to_owned());
        log::info!("Proceeding to route detected root: {:?}", root);
        Some(M::ChangePage(
            match (root.as_ref(), potential_id.as_ref().map(String::as_str)) {
                // TODO convert next two patterns into or-patterns when the feature is implemented
                ("home", None) | ("home", Some("")) => Location::Listing(listing::S {
                    query: url.search.map(PostQuery::Raw),
                }),
                ("posts", Some(id)) => {
                    let marker: model::PostMarker = id.into();
                    Location::Viewer(marker.into())
                }
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
            },
        ))
    }
}
impl From<seed::Url> for RouteMatch {
    fn from(url: seed::Url) -> Self {
        Self(Self::msg_from_url(url))
    }
}
