use seed::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    locations::{Location, viewer::S},
    messages::M as GlobalM,
    model::{PostMarker, Store as GlobalS, StoreOpResult as GSOpResult, StoreOperations as GSOp},
    shared,
};
use db_models::posts;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum M {}

pub fn update(m: M, _s: &mut S, _gs: &GlobalS, _orders: &mut impl Orders<M, GlobalM>) {
    match m {
        // M:: => {}
    }
}