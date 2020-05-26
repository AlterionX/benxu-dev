use seed::browser::url::Url;
use serde::{Deserialize, Serialize};

use crate::model::PostMarker;

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
        Url::new().set_path(&["blog", "posts", self.post_marker.to_string().as_str()])
    }
}