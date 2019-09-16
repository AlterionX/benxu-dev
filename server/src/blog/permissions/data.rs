//! Data structures representing specific transactions.

use serde::{Serialize, Deserialize};

/// Represents a query for looking up permissions in the database. TODO this is really bad, so it
/// would be nice if we make it a kind of permission instead of a permission id.
#[derive(Serialize, Deserialize)]
pub struct Query {
    /// The user id the permissions being searched for belongs to.
    ///
    /// If omitted and `permission_ids` is not omitted, then attempts to find all instances of
    /// permissions_ids regardless of the user.
    ///
    /// If both are omitted, then we find nothing.
    user_id: Option<uuid::Uuid>,
    /// The user id the permissions being searched for belongs to.
    ///
    /// If omitted and `user_id` is not omitted, then attempts to find all instances of
    /// permissions matching `user_id` regardless of permission id.
    ///
    /// If both are omitted, then we find nothing.
    permission_ids: Option<Vec<uuid::Uuid>>,
}
impl Query {
    /// Copies the `user_id` in [`Query`].
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user_id
    }
    /// Borrows the `permissions_ids` in [`Query`].
    pub fn permission_ids(&self) -> Option<&[uuid::Uuid]> {
        self.permission_ids.as_ref().map(|pp| pp.as_slice())
    }
}

