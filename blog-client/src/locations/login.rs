#[derive(Clone)]
pub enum M {
    UserName(String),
    Password(String),
    FormSubmission,
}

use seed::prelude::*;
use futures::Future;
use seed::fetch::FetchObject;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use db_models::models::posts;
use crate::{
    messages::M as GlobalM,
    locations::*,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    username: String,
    password: String,
}

