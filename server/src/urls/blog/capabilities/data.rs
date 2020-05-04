//! Data structures representing specific transactions.

use serde::{Deserialize, Serialize};

/// Represents a query for looking up capabilities in the database. TODO this is really bad, so it
/// would be nice if we make it a kind of capability instead of a capability id.
#[derive(Serialize, Deserialize)]
pub struct Query {
    /// The user id the capabilities being searched for belongs to.
    ///
    /// If omitted and `capability_ids` is not omitted, then attempts to find all instances of
    /// capabilities_ids regardless of the user.
    ///
    /// If both are omitted, then we find nothing.
    user_id: Option<uuid::Uuid>,
    /// The user id the capabilities being searched for belongs to.
    ///
    /// If omitted and `user_id` is not omitted, then attempts to find all instances of
    /// capabilities matching `user_id` regardless of capability id.
    ///
    /// If both are omitted, then we find nothing.
    capability_ids: Option<Vec<uuid::Uuid>>,
}
impl Query {
    /// Copies the `user_id` in [`Query`].
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user_id
    }
    /// Borrows the `capabilities_ids` in [`Query`].
    pub fn capability_ids(&self) -> Option<&[uuid::Uuid]> {
        self.capability_ids.as_ref().map(|pp| pp.as_slice())
    }
}
