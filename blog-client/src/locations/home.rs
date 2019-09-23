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
    shared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub query: Option<PostQuery>,
}

fn render_post(p: &posts::BasicData) -> seed::dom_types::Node<GlobalM> {
    li![
        p![p.published_at
            .map(|d| d.to_string())
            .unwrap_or("Unpublished".to_owned())
        ],
        a![
            attrs!{
                At::Href => format!("/blog/posts/{}", p.id),
            },
            p.title,
        ],
        // self.author.to_view(), // TODO
    ]
}
fn render_post_list(posts: &[posts::BasicData], s: &Store, gs: &GlobalStore) -> seed::dom_types::Node<GlobalM> {
    ul![
        posts
            .iter()
            .filter(|p| (p.published_at.is_some() || gs.user.as_ref().map(|u| u.can_see_unpublished).unwrap_or(false)))
            .map(render_post)
    ]
}
pub fn render(s: &Store, gs: &GlobalStore) -> Vec<seed::dom_types::Node<GlobalM>> {
    if let Some(posts) = gs.posts.as_ref() {
        vec![
            h1![ "Posts" ],
            render_post_list(posts.as_slice(), s, gs),
        ]
    } else {
        vec![shared::views::loading()]
    }
}
