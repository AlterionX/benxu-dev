use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{Location, viewer::M},
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared,
};
use db_models::posts;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct S {
    pub post_marker: PostMarker,
}
impl From<PostMarker> for S {
    fn from(s: PostMarker) -> Self {
        Self { post_marker: s }
    }
}
impl S {
    pub fn to_url(&self) -> Url {
        Url::new(vec!["blog", "posts", self.post_marker.to_string().as_str()])
    }
}