//! Represents all methods to sign into the site.

use serde::{Deserialize, Serialize};

/// Data needed for single sign on (sso). Currently unimplemented
pub mod sso {
    use serde::{Deserialize, Serialize};

    /// An enum matching all the different SSO providers.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Credentials {
        /// Google's SSO provider's data.
        Google(google::Data),
        /// Facebook's SSO provider's data.
        Facebook,
        /// LinkedIn's SSO provider's data.
        LinkedIn,
        /// Twitter's SSO provider's data.
        Twitter,
        /// Microsoft's SSO provider's data.
        Microsoft,
        /// Github's SSO provider's data.
        Github,
    }

    /// Google specific SSO structs.
    pub mod google {
        #[cfg(feature = "diesel")]
        use crate::schema::*;
        use serde::{Deserialize, Serialize};

        /// The complete model of a row in the `google_sso` table.
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[cfg_attr(
            feature = "diesel",
            derive(Identifiable, Queryable),
            table_name = "google_sso"
        )]
        pub struct Data {
            /// The id of the row.
            pub id: uuid::Uuid,
            /// The id of the user this belongs to.
            pub user_id: uuid::Uuid,
        }
    }
}

/// Password record data.
pub mod pw {
    #[cfg(feature = "diesel")]
    use crate::schema::*;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// Fully represents a row in the passwords table.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(
        feature = "diesel",
        derive(Identifiable, Associations, Queryable),
        belongs_to(parent = "crate::models::users::Data", foreign_key = "user_id"),
        table_name = "passwords"
    )]
    pub struct Data {
        /// Id of the row.
        pub id: uuid::Uuid,
        /// Time the row was created.
        pub created_at: DateTime<Utc>,
        /// Who created the row.
        pub created_by: uuid::Uuid,
        /// Last time this row was updated.
        pub updated_at: DateTime<Utc>,
        /// Who updated the row.
        pub updated_by: uuid::Uuid,
        /// The id of the user this password belongs to.
        pub user_id: uuid::Uuid,
        /// A hash of the password.
        pub hash: String,
        /// The salt used when hashing the password.
        pub salt: String,
    }

    /// Represents a new row to be added to the table.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(feature = "diesel", derive(Insertable), table_name = "passwords")]
    pub struct NewWithId<'a> {
        /// Id of the row to be added.
        id: uuid::Uuid,
        /// Id of the user who will create the row.
        created_by: uuid::Uuid,
        /// Id of the user who most recently updated (aka creeated) the row.
        updated_by: uuid::Uuid,
        /// The id of the user this password belongs to.
        user_id: uuid::Uuid,
        /// A hash of the password.
        hash: &'a str,
        /// The salt used when hashing the password.
        salt: &'a str,
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "server")]
    impl<'a> From<New<'a>> for NewWithId<'a> {
        fn from(new: New<'a>) -> Self {
            Self {
                id: uuid::Uuid::new_v4(),
                created_by: new.created_by,
                updated_by: new.updated_by,
                user_id: new.user_id,
                hash: new.hash,
                salt: new.salt,
            }
        }
    }

    /// Represents a new row without the primary key.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct New<'a> {
        /// Id of the user who will create the row.
        pub created_by: uuid::Uuid,
        /// Id of the user who most recently updated (aka creeated) the row.
        pub updated_by: uuid::Uuid,
        /// The id of the user this password belongs to.
        pub user_id: uuid::Uuid,
        /// A hash of the password.
        pub hash: &'a str,
        /// The salt used when hashing the password.
        pub salt: &'a str,
    }

    /// Represents a set of changes to the row.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(feature = "diesel", derive(AsChangeset), table_name = "passwords")]
    pub struct Changed {
        /// Id of the user who most recently updated (aka creeated) the row.
        pub updated_by: uuid::Uuid,
        /// A hash of the password.
        pub hash: Option<String>,
        /// The salt used when hashing the password.
        pub salt: Option<String>,
    }
}

/// FIDO records. Currently unimplemented.
pub mod fido {}

/// Represents one of many types of credentials stored in database.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Data {
    /// Denotes that data represents a password credential.
    Password(pw::Data),
}
impl From<pw::Data> for Data {
    fn from(data: pw::Data) -> Self {
        Self::Password(data)
    }
}
impl Data {
    /// Unifies each respective type as a credential.
    pub fn from_result<E>(result_data: Result<pw::Data, E>) -> Result<Self, E> {
        result_data.map(|data| Self::from(data))
    }
}
