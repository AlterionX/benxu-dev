use std::fmt::Display;

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use crate::{
    model::Store as GlobalS,
    messages::M as GlobalM,
};

#[derive(Debug, Clone)]
pub enum M {}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct S {
    post_marker: String,
}
impl From<String> for S {
    fn from(s: String) -> Self {
        Self {
            post_marker: s,
        }
    }
}

pub fn update(m: M, s: &mut S, gs: &GlobalS, orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}
pub fn render(s: &S, gs: &GlobalS) -> impl View<M> {
    p![]
}

