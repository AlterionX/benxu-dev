
use crate::{
    blog::{
        accounts,
        auth,
    },
};

pub enum Login {
    UserCreate(accounts::error::Create),
    Authentication(auth::Error),
}

