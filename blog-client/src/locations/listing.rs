use seed::prelude::*;
use serde::{Serialize, Deserialize};

use db_models::models::posts;
use crate::{
    model::{Store as GlobalS, StoreOperations as GSOp, StoreOpResult as GSOpResult},
    messages::{M as GlobalM, AsyncM as GlobalAsyncM},
    requests::PostQuery,
    shared,
    locations::Location,
};

pub fn data_load(s: S, gs: &GlobalS) -> impl GlobalAsyncM {
    use seed::fetch::Request;
    const POSTS_URL: &'static str = "/api/posts";
    let query = s.query.unwrap_or_else(PostQuery::default);
    let url = format!("{}?{}", POSTS_URL, query);
    // TODO figure out caching and determing if data already loaded instead of going
    // straight to server all the time.
    Request::new(url).fetch_json(|fo| GlobalM::StoreOpWithAction(
        GSOp::PostListing(query, fo),
        after_fetch,
    ))
}
fn after_fetch(_gs: *const GlobalS, res: GSOpResult) -> Option<GlobalM> {
    use GSOpResult::*;
    match res {
        Success => Some(GlobalM::RenderPage(Location::Listing(S { query: None }))),
        Failure(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum M {}
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
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

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}
fn render_post(p: &posts::BasicData) -> seed::dom_types::Node<M> {
    log::debug!("Not called");
    li![
        p![p.published_at
            .map(|d| d.to_string())
            .unwrap_or("Unpublished".to_owned())
        ],
        a![
            attrs!{ At::Href => if p.is_published() {
                format!("/blog/posts/{}", p.id)
            } else {
                format!("/blog/editor/{}", p.id)
            } },
            p.title,
        ],
        // self.author.to_view(), // TODO
    ]
}
fn render_post_list(posts: &[posts::BasicData], s: &S, gs: &GlobalS) -> seed::dom_types::Node<M> {
    if posts.is_empty() {
        log::debug!("Calling render_post_list.");
        p!["Coming soon!"]
    } else {
        log::debug!("Not called");
        ul![
            posts
                .iter()
                .map(render_post)
        ]
    }
}
pub fn render(s: &S, gs: &GlobalS) -> Vec<seed::dom_types::Node<M>> {
    vec![
        vec![h1![ "Posts" ]],
        if let Some(posts) = gs.published_posts.as_ref() {
            log::debug!("Calling published render.");
            vec![render_post_list(posts.as_slice(), s, gs)]
        } else {
            log::debug!("Not called");
            vec![shared::views::loading()]
        },
        match gs {
            GlobalS { user: Some(user), unpublished_posts: Some(posts), .. } if user.can_see_unpublished => {
                log::debug!("Calling unpublished render.");
                vec![
                    h1!["Unpublished Drafts"],
                    render_post_list(posts.as_slice(), s, gs),
                ]
            },
            _ => vec![],
        },
    ].into_iter().flatten().collect()
}

