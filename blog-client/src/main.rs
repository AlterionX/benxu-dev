use wasm_bindgen::prelude::*;
use chrono::{DateTime, Utc};

struct Post {
    post: String,
    author: String,
    date: DateTime<Utc>,
}

struct Model {
    post: Option<Post>,
}

enum Msg {
    Login,
    SubmitPost,
    AccessPost,
}

#[wasm_bindgen]
pub fn render() {
}

fn main() {
}
