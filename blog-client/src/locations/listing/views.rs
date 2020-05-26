use seed::prelude::*;

use crate::{
    locations::listing::{M, S},
    model::{Name, Store as GlobalS},
    shared,
};
use db_models::models::posts;

fn render_post(p: &posts::BasicData, author: Option<&Name>) -> Node<M> {
    log::debug!("Not called");
    li![
        attrs! {
            At::Class => "post-item";
        },
        h2![
            attrs! { At::Class => "as-h3" },
            a![
                attrs! {
                    At::Href => if p.is_published() {
                        format!("/blog/posts/{}", p.id)
                    } else {
                        format!("/blog/editor/{}", p.id)
                    };
                    At::Class => "post-title-link";
                },
                p.title.as_str(),
            ],
        ],
        p![
            attrs! { At::Class => "post-published-date" },
            p.published_at
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Unpublished".to_owned())
        ],
        author.map_or_else(|| empty![], |n| n.to_view()), // TODO
    ]
}
fn render_post_list(empty_msg: &str, posts: &[posts::BasicData]) -> Node<M> {
    if posts.is_empty() {
        log::debug!("Calling render_post_list.");
        p![attrs! {At::Class => "no-post-text"}, empty_msg,]
    } else {
        log::debug!("Not called");
        ul![
            posts
                .iter()
                .map(|p| -> Node<M> { render_post(p, None) }) // TODO load authors
        ]
    }
}
pub fn render_post_pagination_buttons(s: &S) -> Node<M> {
    // TODO
    div![
        attrs! {
            At::Class => "pagination-buttons";
        },
        match s.generate_next_url() {
            Some(url) => a![
                attrs! {
                    At::Class => "next";
                    At::Href => url;
                },
                "Next >"
            ],
            None => empty![],
        },
        match s.generate_prev_url() {
            Some(url) => a![
                attrs! {
                    At::Class => "prev";
                    At::Href => url;
                },
                "< Previous"
            ],
            None => empty![],
        },
    ]
}
pub fn render(s: &S, gs: &GlobalS) -> Vec<Node<M>> {
    vec![
        div![
            attrs! {
                At::Class => "post-list";
            },
            h1!["Posts"],
            if let Some(posts) = gs.published_posts.as_ref() {
                log::debug!("Calling published render.");
                vec![render_post_list("Coming soon.", posts.as_slice())]
            } else {
                log::debug!("No posts found.");
                vec![shared::views::loading()]
            },
        ],
        match gs {
            GlobalS {
                user: Some(user),
                unpublished_posts: Some(posts),
                ..
            } if user.can_see_unpublished => {
                log::debug!("Calling unpublished render.");
                div![
                    attrs! {
                        At::Class => "unpublished-post-list";
                    },
                    h1!["Unpublished Drafts"],
                    render_post_list("None found.", posts.as_slice()),
                ]
            }
            _ => empty![],
        },
        render_post_pagination_buttons(s),
    ]
}
