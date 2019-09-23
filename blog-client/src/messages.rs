use boolinator::Boolinator;
use crate::{
    requests::PostQuery,
    locations::*,
};

#[derive(Clone)]
pub enum PostAccessMethod {
    ById(uuid::Uuid),
    ByShortName(String),
}
#[derive(Clone)]
pub enum M {
    Login(login::M),
    SubmitPost(String, String, Option<uuid::Uuid>),
    AccessPost(PostAccessMethod),
    ChangePage(Location),
    RenderPage(Location),
    LoadInternal,
    Navigate,
    DataFetched(LocationWithData),
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
    fn to_opt_msg(url: seed::Url) -> Option<M> {
        crate::log("Hello from beyond...");
        let root = (url.path.get(0)?.as_ref(): &str == "blog")
            .as_some_from(|| url.path.get(1))?
            .map(|s| if s == "" {
                "home"
            } else {
                s.as_str()
            })
            .unwrap_or("home");
        crate::log(format!("{:?}", root).as_str());
        Some(M::ChangePage(match root {
            "home" => Location::Home(home::Store { query: url.search.map(PostQuery::Raw) }),
            "posts" => Location::Viewer(url.path[2].as_str().into()),
            "editor" => Location::Editor(url.path[2].as_str().into()),
            "login" => Location::Login(login::Store::default()),
            _ => Location::NotFound,
        }))
    }
}
impl From<seed::Url> for RouteMatch {
    fn from(url: seed::Url) -> Self {
        Self(Self::to_opt_msg(url))
    }
}
