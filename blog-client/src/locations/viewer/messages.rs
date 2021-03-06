use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::viewer::S,
    messages::M as GlobalM,
    model::Store as GlobalS,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {}

pub fn update(m: M, _s: &mut S, _gs: &GlobalS, _orders: &mut impl Orders<GlobalM, GlobalM>) {
    match m {
        // M:: => {}
    }
}