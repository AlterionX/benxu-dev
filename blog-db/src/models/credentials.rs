mod sso {
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

    mod google {
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

mod pw {
    use crate::schema::*;
    use serde::{
        Serialize,
        Deserialize,
    };
    #[derive(Identifiable, Queryable, Serialize, Deserialize)]
    #[table_name="passwords"]
    pub struct Data {
        pub id: uuid::Uuid,
        pub user_id: String,
        pub hash: String,
    }

    #[derive(Insertable, Serialize, Deserialize)]
    #[table_name="passwords"]
    pub struct NewWithId<'a> {
        id: uuid::Uuid,
        hash: &'a str,
        user_id: uuid::Uuid,
        created_by: uuid::Uuid,
    }
    impl<'a> From<New<'a>> for NewWithId<'a> {
        fn from(new: New<'a>) -> Self {
            Self {
                id: uuid::Uuid::new_v4(),
                hash: new.hash,
                user_id: new.user_id,
                created_by: new.created_by,
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct New<'a> {
        hash: &'a str,
        user_id: uuid::Uuid,
        created_by: uuid::Uuid,
    }

    #[derive(AsChangeset, Serialize, Deserialize)]
    #[table_name="passwords"]
    pub struct Changed {
        pub hash: Option<String>,
    }
}

mod fido {
}


