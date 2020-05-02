use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::Location,
    messages::{M as GlobalM},
    model::{Name, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    requests::PostQuery,
    shared,
};
use db_models::models::posts;

pub async fn data_load(s: S) -> Result<GlobalM, GlobalM> {
    use seed::fetch::Request;
    const POSTS_URL: &str = "/api/posts";
    let query = s.query.unwrap_or_else(PostQuery::default);
    let url = format!("{}?{}", POSTS_URL, query);
    // TODO figure out caching and determing if data already loaded instead of going
    // straight to server all the time.
    Request::new(url)
        .fetch_json(|fo| GlobalM::StoreOpWithAction(GSOp::PostListing(query, fo), after_fetch))
        .await
}
fn after_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::RenderPage(Location::Listing(S { query: None }))),
        Failure(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {}
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct S {
    pub query: Option<PostQuery>,
}
impl S {
    pub fn to_url(&self) -> Url {
        let mut url = Url::new(vec!["blog"]);
        if let Some(q) = &self.query {
            url = url.search(format!("{}", q).as_str());
        }
        url
    }
}

pub fn update(m: M, _s: &mut S, _gs: &GlobalS, _orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}
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
                p.title,
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
        p![
            attrs!{At::Class => "no-post-text"},
            empty_msg,
        ]
    } else {
        log::debug!("Not called");
        ul![
            posts
                .iter()
                .map(|p| -> Node<M> { render_post(p, None) }) // TODO load authors
        ]
    }
}
pub fn render(_s: &S, gs: &GlobalS) -> Vec<Node<M>> {
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
                log::debug!("Not called");
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
    ]
}
