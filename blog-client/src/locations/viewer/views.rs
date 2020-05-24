use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{Location, viewer::{M, S}},
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared,
};
use db_models::posts;

pub fn render(s: &S, gs: &GlobalS) -> Node<M> {
    match gs.post.as_ref() {
        Some(post) if s.post_marker.refers_to(post) => render_post(post),
        _ => shared::views::loading(),
    }
}
fn render_post(post: &posts::DataNoMeta) -> Node<M> {
    div![
        attrs! { At::Class => "post" },
        h1![post.title],
        md![post.body.as_str()],
    ]
}