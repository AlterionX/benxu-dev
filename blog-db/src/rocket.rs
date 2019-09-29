use crate::query::DBConn;

use rocket::{fairing, http, logger, request, Outcome, Rocket, State};
#[rocket_contrib::database("blog")]
pub struct DB(diesel::prelude::PgConnection);

/// Logistics implementation.
impl DBConn for DB {
    /// Access a reference to the connection actually used to connect to the DB. Deref gives the
    /// actual struct [`PgConnection`].
    fn conn(&self) -> &diesel::prelude::PgConnection {
        &self.0
    }
}
