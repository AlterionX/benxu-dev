use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use db_models::models::posts;
use crate::{
    model::Store as GlobalS,
    messages::M as GlobalM,
    requests::PostQuery,
    shared,
};

#[derive(Debug, Clone)]
pub enum M {}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct S {
    pub query: Option<PostQuery>,
}

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}
fn render_post(p: &posts::BasicData) -> seed::dom_types::Node<M> {
    li![
        p![p.published_at
            .map(|d| d.to_string())
            .unwrap_or("Unpublished".to_owned())
        ],
        a![
            attrs!{ At::Href => format!("/blog/posts/{}", p.id) },
            p.title,
        ],
        // self.author.to_view(), // TODO
    ]
}
fn render_post_list(posts: &[posts::BasicData], s: &S, gs: &GlobalS) -> seed::dom_types::Node<M> {
    ul![
        posts
            .iter()
            .filter(|p| (p.published_at.is_some() || gs.user.as_ref().map(|u| u.can_see_unpublished).unwrap_or(false)))
            .map(render_post)
    ]
}
pub fn render(s: &S, gs: &GlobalS) -> Vec<seed::dom_types::Node<M>> {
    if let Some(posts) = gs.posts.as_ref() {
        vec![
            h1![ "Posts" ],
            render_post_list(posts.as_slice(), s, gs),
        ]
    } else {
        vec![shared::views::loading()]
    }
}
