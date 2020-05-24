use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{editor::S, Location},
    messages::M as GlobalM,
    model::Store as GlobalS,
};
use db_models::models::*;

fn update_post(to_update: &mut posts::DataNoMeta, updated: &posts::DataNoMeta) {
    to_update.created_by = updated.created_by;
    to_update.created_at = updated.created_at;
    to_update.published_by = updated.published_by;
    to_update.published_at = updated.published_at;
    to_update.archived_by = updated.archived_by;
    to_update.archived_at = updated.archived_at;
    to_update.deleted_by = updated.deleted_by;
    to_update.deleted_at = updated.deleted_at;
    to_update.title = updated.title.clone();
    to_update.body = updated.body.clone();
    to_update.slug = updated.slug.clone();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {
    Title(String),
    Body(String),
    Slug(String),
    Publish,
    Save,

    SyncPost,
}

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    use M::*;
    match s {
        S::Undetermined(_) => return,
        _ => (),
    };
    match m {
        Title(title) => s.update_title(title),
        Body(body) => s.update_body(body),
        Slug(slug) => s.update_slug(slug),
        Publish => {
            if let Some(user) = gs.user.as_ref() {
                if let Some(req) = s.attempt_publish(user) {
                    orders.perform_g_cmd(req);
                } else {
                    log::error!("Failed to create publish request.")
                }
            } else {
                log::error!("Attempted publish while not logged in.")
            }
        }
        Save => {
            if let Some(req) = s.attempt_save() {
                orders.perform_g_cmd(req);
            } else {
                log::error!("Failed to create save request.");
            }
        }

        SyncPost => {
            if let Some(updated) = &gs.post {
                match s {
                    S::Old(post, _) if post.id == updated.id => update_post(post, updated),
                    _ => {
                        orders.send_g_msg(GlobalM::ChangePageAndUrl(Location::Editor(S::Old(
                            updated.clone(),
                            posts::Changed::default(),
                        ))));
                    }
                }
            } else {
                log::warn!("Attempted to sync with nonexistent post.");
            }
        }
    }
}
