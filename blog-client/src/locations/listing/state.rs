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
        Url::new(vec!["blog"])
    }

    pub fn generate_url(q: &PostQuery) -> Url {
        Self::url_root().search(format!("{}", q).as_str())
    }

    pub fn generate_next_url(&self) -> Option<Url> {
        self.query
            .unwrap_or(Default::default())
            .generate_next()
            .as_ref()
            .map(Self::generate_url)
    }

    pub fn generate_prev_url(&self) -> Option<Url> {
        self.query
            .unwrap_or(Default::default())
            .generate_prev()
            .as_ref()
            .map(Self::generate_url)
    }
}
