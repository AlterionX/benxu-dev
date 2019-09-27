use boolinator::Boolinator;
use crate::{
    requests::PostQuery,
    locations::*,
    model,
};

#[derive(Clone)]
pub enum PostAccessMethod {
    ById(uuid::Uuid),
    ByShortName(String),
}
#[derive(Clone)]
pub enum M {
    // Change locations
    ChangePage(Location),
    RenderPage(Location),
    // Globabl state
    StoreOp(model::StoreOperations, fn(Result<Option<Location>, ()>) -> Option<M>),
    // Location specific
    Login(login::M),
    Editor(editor::M),
    Viewer(viewer::M),
    Listing(listing::M),
}
impl M {
    pub fn page_change(loc: Location) -> Self {
        Self::ChangePage(loc)
    }
}

pub struct RouteMatch(Option<M>);
impl RouteMatch {
    pub fn into_inner(self) -> Option<M> {
        self.0
    }
    fn to_opt_msg(mut url: seed::Url) -> Option<M> {
        crate::log("Hello from beyond...");
        let potential_id = (url.path.len() >= 3)
            .as_some_from(|| url.path.remove(2));
        let potential_resource = (url.path.len() >= 2)
            .as_some_from(|| url.path.remove(1));
        let potential_root = (url.path.len() >= 1)
            .as_some_from(|| url.path.remove(0));
        let root = (potential_root?.as_ref(): &str == "blog")
            .as_some_from(|| potential_resource)?
            .map(|s| if s == "" { "home".to_owned() } else { s })
            .unwrap_or("home".to_owned());
        crate::log(format!("{:?}", root).as_str());
        Some(M::ChangePage(match root.as_ref() {
            "home" => Location::Listing(listing::S { query: url.search.map(PostQuery::Raw) }),
            "posts" => Location::Viewer(potential_id.expect("post_id to be present").into()),
            "editor" => Location::Editor(potential_id.expect("post_id to be present").into()),
            "login" => Location::Login(login::S::default()),
            _ => Location::NotFound,
        }))
    }
}
impl From<seed::Url> for RouteMatch {
    fn from(url: seed::Url) -> Self {
        Self(Self::to_opt_msg(url))
    }
}
