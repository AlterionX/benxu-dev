use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::requests::PostQuery;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct S {
    pub query: Option<PostQuery>,
}
impl S {
    pub fn to_url(&self) -> Url {
        if let Some(q) = &self.query {
            Self::generate_url(q)
        } else {
            Self::url_root()
        }
    }

    pub fn url_root() -> Url {
        Url::new().set_path(vec!["blog"])
    }

    pub fn generate_url(q: &PostQuery) -> Url {
        Self::url_root().set_search(q)
    }

    pub fn generate_next_url(&self) -> Option<Url> {
        let q = self.query
            .as_ref()
            .ok_or_else(|| PostQuery::default());
        let q_ref = match &q {
            Err(e) => e,
            Ok(q) => *q,
        };
        q_ref
            .generate_next()
            .as_ref()
            .map(Self::generate_url)
    }

    pub fn generate_prev_url(&self) -> Option<Url> {
        let q = self.query
            .as_ref()
            .ok_or_else(|| PostQuery::default());
        let q_ref = match &q {
            Err(e) => e,
            Ok(q) => *q,
        };
        q_ref
            .generate_prev()
            .as_ref()
            .map(Self::generate_url)
    }
}
