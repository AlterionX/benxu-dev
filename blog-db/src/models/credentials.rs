use crate::schema::*;
use serde::{
    Serialize,
    Deserialize,
};

mod sso {
    use crate::schema::*;
    use serde::{
        Serialize,
        Deserialize,
    };
    #[derive(Serialize, Deserialize)]
    pub enum Provider {
        Google,
        Facebook,
        LinkedIn,
        Twitter,
        Microsoft,
        Github,
    }
    #[derive(Serialize, Deserialize)]
    pub struct Credentials {
        provider: Provider,
    }
}

mod pw {
    use crate::schema::*;
    use serde::{
        Serialize,
        Deserialize,
    };
    #[derive(Queryable, Serialize, Deserialize)]
    #[table_name="passwords"]
    struct UserAndPassword {
        password: String,
        user: String,
    }
    #[derive(AsChangeset, Serialize, Deserialize)]
    #[table_name="passwords"]
    struct Password {
        password_hash: String,
    }
}

mod fido {
    use crate::schema::*;
    use serde::{
        Serialize,
        Deserialize,
    };
}


