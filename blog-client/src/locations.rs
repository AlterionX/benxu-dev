use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    messages::{AsyncM as GlobalAsyncM, M as GlobalM},
    model::{Store as GlobalS, StoreOperations as GSOp},
};

pub mod editor;
pub mod listing;
pub mod login;
pub mod viewer;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Location {
    Login(login::S),
    Viewer(viewer::S),
    Listing(listing::S),
    Editor(editor::S),
    Logout,
    NotFound,
}
impl Default for Location {
    fn default() -> Self {
        Self::NotFound
    }
}
impl Location {
    pub fn find_redirect(self, gs: &GlobalS) -> Result<Self, Self> {
        match &self {
            Location::Login(_) if gs.user.is_some() => Ok(Location::Listing(listing::S::default())),
            // TODO Editor hops to not found. Pressing back will load the editor properly.
            Location::Editor(s) if editor::is_restricted_from(s, gs) => Ok(Location::NotFound),
            Location::Viewer(_) if viewer::is_restricted_from(gs) => Ok(Location::NotFound),
            _ => Err(self),
        }
    }
    pub fn fetch_req(self, gs: &GlobalS) -> Result<std::pin::Pin<Box<dyn GlobalAsyncM>>, Self> {
        match self {
            Location::Listing(store) => Ok(Box::pin(listing::data_load(store))),
            Location::Logout => Ok(Box::pin(login::logout_trigger())),
            Location::Editor(editor::S::Undetermined(post_id)) if gs.has_cached_post(&post_id) => {
                Err(Location::Editor(editor::S::Old(
                    gs.post.as_ref().unwrap().clone(),
                    Default::default(),
                )))
            }
            Location::Editor(editor::S::Undetermined(post_id)) if !gs.has_cached_post(&post_id) => {
                Ok(Box::pin(editor::load_post(post_id.clone())))
            }
            Location::Viewer(viewer::S {
                post_marker: pm, ..
            }) => {
                if gs.has_cached_post(&pm) {
                    Err(Location::Viewer(pm.into()))
                } else {
                    Ok(Box::pin(viewer::load_post(pm.clone())))
                }
            }
            _ => Err(self),
        }
    }
    fn get_pre_load_messages(&self, _gs: &GlobalS) -> Option<GlobalM> {
        match self {
            _ => None,
        }
    }
    pub fn to_url(&self) -> Url {
        match self {
            Self::Login(s) => s.to_url(),
            Self::Viewer(s) => s.to_url(),
            Self::Listing(s) => s.to_url(),
            Self::Editor(s) => s.to_url(),
            Self::Logout => Url::new(vec!["blog", "logout"]),
            Self::NotFound => Url::new(vec!["blog", "404"]),
        }
    }
}
impl Location {
    pub fn prep_page_for_render(
        self,
        _prev: &Location,
        gs: &GlobalS,
        orders: &mut impl Orders<GlobalM, GlobalM>,
    ) {
        self.get_pre_load_messages(gs).map(|m| orders.send_msg(m));
        let loc = match self.find_redirect(&gs) {
            Ok(redirect) => {
                log::trace!("Attempt to redirect to another page.");
                orders.skip().send_msg(GlobalM::ChangePageAndUrl(redirect));
                return;
            }
            Err(loc) => loc,
        };
        match loc.fetch_req(gs) {
            Ok(req) => {
                log::trace!("Attempt to fetch data.");
                orders.skip().perform_cmd(req);
            }
            Err(loc) => {
                log::trace!("Attempt to render page directly, since data is already present.");
                orders.skip().send_msg(GlobalM::RenderPage(loc));
            }
        }
    }
    pub fn post_load_msgs(&self) -> Option<GlobalM> {
        Some(match self {
            Location::Login(_) => GlobalM::Login(login::M::SetFocus),
            _ => None?,
        })
    }
    pub fn to_view(&self, gs: &GlobalS) -> Vec<Node<GlobalM>> {
        match self {
            Location::Logout => vec![h1!["Logging out..."]],
            Location::Listing(s) => listing::render(s, gs).map_msg(GlobalM::Listing),
            Location::Login(s) => vec![login::render(s, gs).map_msg(GlobalM::Login)],
            Location::Viewer(s) => vec![viewer::render(s, gs).map_msg(GlobalM::Viewer)],
            Location::Editor(s) => editor::render(s, gs).map_msg(GlobalM::Editor),
            Location::NotFound => vec![p!["Page not found!"]],
        }
    }
}
