pub mod sso {
    use serde::{
        Serialize,
        Deserialize,
    };
    #[derive(Serialize, Deserialize)]
    pub enum Credentials {
        Google(google::Data),
        Facebook,
        LinkedIn,
        Twitter,
        Microsoft,
        Github,
    }

    pub mod google {
        use crate::schema::*;
        use serde::{
            Serialize,
            Deserialize,
        };

        #[derive(Identifiable, Queryable, Serialize, Deserialize)]
        #[table_name="google_sso"]
        pub struct Data {
            pub id: uuid::Uuid,
            pub user_id: uuid::Uuid,
        }
    }
}

pub mod pw {
    use crate::{schema::*, models};
    use serde::{
        Serialize,
        Deserialize,
    };
    use chrono::{DateTime, Utc};

    #[derive(Identifiable, Associations, Queryable, Serialize, Deserialize)]
    #[belongs_to(parent = "models::users::Data", foreign_key = "user_id")]
    #[table_name="passwords"]
    pub struct Data {
        pub id: uuid::Uuid,
        pub created_at: DateTime<Utc>,
        pub created_by: uuid::Uuid,
        pub updated_at: DateTime<Utc>,
        pub updated_by: uuid::Uuid,
        pub user_id: uuid::Uuid,
        pub hash: String,
        pub salt: String,
    }

    #[derive(Insertable, Serialize, Deserialize)]
    #[table_name="passwords"]
    pub struct NewWithId<'a> {
        id: uuid::Uuid,
        created_by: uuid::Uuid,
        updated_by: uuid::Uuid,
        user_id: uuid::Uuid,
        hash: &'a str,
        salt: &'a str,
    }
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

    #[derive(Serialize, Deserialize)]
    pub struct New<'a> {
        pub created_by: uuid::Uuid,
        pub updated_by: uuid::Uuid,
        pub user_id: uuid::Uuid,
        pub hash: &'a str,
        pub salt: &'a str,
    }

    #[derive(AsChangeset, Serialize, Deserialize)]
    #[table_name="passwords"]
    pub struct Changed {
        pub hash: Option<String>,
        pub salt: Option<String>,
    }
}

pub mod fido {
}

/// Represents one of many types of credentials stored in database.
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
    pub fn from_result<E>(result_data: Result<pw::Data, E>) -> Result<Self, E> {
        result_data.map(|data| Self::from(data))
    }
}

