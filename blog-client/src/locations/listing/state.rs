use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::requests::PostQuery;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct S {
    pub query: Option<PostQuery>,
}
impl S {
    pub fn to_url(&self) -> Url {
        let mut url = ;
        if let Some(q) = &self.query {
            Self::query_url(q);
        } else {
            Self::url_root()
        }
        url
    }

    pub fn url_root() -> Url {
        Url::new(vec!["blog"])
    }

    pub fn generate_url(post: &PostQuery) -> Url {
        Self::url_root().search(format!("{}", q).as_str())
    }

    pub fn generate_next_url(&self) -> Option<Url> {
        self.query.as_ref()
            .and_then(PostQuery::generate_next)
            .map(Self::generate_url)
    }

    pub fn generate_prev_url(&self) -> Option<Url> {
        self.query.as_ref()
            .and_then(PostQuery::generate_prev)
            .map(Self::generate_url)
    }
}
