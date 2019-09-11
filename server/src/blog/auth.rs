use std::{
    io::Read,
    sync::Arc,
    marker::PhantomData,
    ops::Deref,
    str,
};
use rocket::{
    response::{
        status,
        Redirect,
    },
    http::{Status, Cookie, Cookies, ContentType},
    request::{
        Request,
        FromRequest,
        Outcome,
    },
    data::{
        Outcome as OutcomeWithData,
        Data,
        FromDataSimple,
    },
    State,
};
use rocket_contrib::json::Json;
use serde::{
    Serialize,
    Deserializer,
    Deserialize,
};

use blog_db::models::*;
use crate::{
    crypto::{
        KeyStore,
        CurrAndLastKey,
        algo::{
            Algo as A,
            hash::{
                symmetric::Algo as HashA,
                argon2::d::{Algo as ARGON2D, SigningData as ARGON2D_MSG},
            },
        },
        token::paseto,
    },
    blog::db as db,
};

/// Used to indicate that a type represents a permissions level. A Custom variant is listed for
/// anything not covered by the enum.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum Permission {
    EditPost,
    CreatePost,
    DeletePost,
    PublishPost,
    ArchivePost,
    CreateUser,
    GrantPermission,
    EditUser,
    EditUserCredentials,
    Custom { name: String },
}
// TODO make string const and lift into enum declaration
impl Permission {
    fn as_str(&self) -> &str {
        match self {
            Self::EditPost => "edit_post",
            Self::CreatePost => "create_post",
            Self::DeletePost => "delete_post",
            Self::PublishPost => "publish_post",
            Self::ArchivePost => "archive_post",
            Self::CreateUser => "create_user",
            Self::GrantPermission => "grant_permission",
            Self::EditUser => "edit_user",
            Self::EditUserCredentials => "edit_user_credentials",
            Self::Custom { name } => &name,
        }
    }
}
// TODO make string const and lift into enum declaration
impl From<&permissions::Data> for Permission {
    fn from(perm: &permissions::Data) -> Self {
        match perm.permission.as_str() {
            "edit_post" => Self::EditPost,
            "create_post" => Self::CreatePost,
            "delete_post" => Self::DeletePost,
            "publish_post" => Self::PublishPost,
            "archive_post" => Self::ArchivePost,
            "create_user" => Self::CreateUser,
            "grant_permission" => Self::GrantPermission,
            "edit_user" => Self::EditUser,
            "edit_user_credentials" => Self::EditUserCredentials,
            p @ _ => Self::Custom { name: p.to_owned() },
        }
    }
}

/// Used to indicate that a type represents a permissions level.
pub trait Verifiable {
    const REQUIRED_PERMS: &'static[Permission];
    /// Verifies that the list of permissions passed in satisfies the constraints of the permission level.
    fn verify<T>(cred: &Credentials<T>) -> bool {
        cred.has_permissions(&Self::REQUIRED_PERMS)
    }
    fn verify_slice(perms: &[Permission]) -> bool {
        for req_perm in Self::REQUIRED_PERMS.iter() {
            if !perms.contains(req_perm) {
                return false;
            }
        }
        true
    }
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to create users.
pub struct CanCreateUser;
impl Verifiable for CanCreateUser {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::CreateUser];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to edit users.
pub struct CanEditUser;
impl Verifiable for CanEditUser {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::EditUser];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to edit users.
pub struct CanEditUserCredentials;
impl Verifiable for CanEditUserCredentials {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::EditUserCredentials];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to edit blog posts.
pub struct CanEdit;
impl Verifiable for CanEdit {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::EditPost];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to delete blog posts.
pub struct CanDelete;
impl Verifiable for CanDelete {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::DeletePost];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to create blog posts.
pub struct CanPost;
impl Verifiable for CanPost {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::CreatePost];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to publish blog posts.
pub struct CanPublish;
impl Verifiable for CanPublish {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::PublishPost];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to archive blog posts.
pub struct CanArchive;
impl Verifiable for CanArchive {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::ArchivePost];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to grant permissions to users. NOTE: Can only grant permissions they already have.
pub struct CanGrantPermission;
impl Verifiable for CanGrantPermission {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::GrantPermission];
}

/// Type to represent the Admin privlege level. This level of privlege simply represents everything
/// in the system.
pub struct Admin;
impl Verifiable for Admin {
    const REQUIRED_PERMS: &'static[Permission] = &[
        Permission::CreateUser,
        Permission::EditPost,
        Permission::CreatePost,
        Permission::DeletePost,
        Permission::PublishPost,
        Permission::ArchivePost,
        Permission::GrantPermission,
    ];
}

/// Type to allow for the verification of a Credentials allowing for arbitrary permissions. Simply
/// a rename of the Unit type to make purpose clearer.
pub type AnyPermissions = ();
impl Verifiable for AnyPermissions {
    const REQUIRED_PERMS: &'static[Permission] = &[];
    fn verify<T>(_: &Credentials<T>) -> bool {
        true
    }
}

/// L for Level
#[derive(Serialize)]
pub struct Credentials<L> {
    #[serde(skip)]
    level: PhantomData<L>,
    permissions: Vec<Permission>,
    user_id: uuid::Uuid,
}
#[derive(Deserialize)]
pub struct UnverifiedPermissionsCredential(Credentials<AnyPermissions>);
impl UnverifiedPermissionsCredential {
    fn new(user_id: uuid::Uuid, perms: Vec<Permission>) -> Self {
        // Should not be able to error
        Self(Credentials::new(user_id, perms).unwrap())
    }
}
impl Deref for UnverifiedPermissionsCredential {
    type Target = Credentials<AnyPermissions>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl <'de> Deserialize<'de> for Credentials<AnyPermissions> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        enum Field {
            Permissions,
            UserId,
            Ignore,
        }
        struct FieldVisitor;
        impl <'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut serde::export::Formatter) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "field identifier")
            }
            fn visit_u64<E>(self, value: u64) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    0u64 => serde::export::Ok(Field::Permissions),
                    1u64 => serde::export::Ok(Field::UserId),
                    _ => serde::export::Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Unsigned(value),
                            &"field index 0 <= i < 2"
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    "permissions" => serde::export::Ok(Field::Permissions),
                    "user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore)
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> serde::export::Result<Self::Value, E> where E: serde::de::Error {
                match value {
                    b"permissions" => serde::export::Ok(Field::Permissions),
                    b"user_id" => serde::export::Ok(Field::UserId),
                    _ => serde::export::Ok(Field::Ignore)
                }
            }
        }
        impl <'de> serde::Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> serde::export::Result<Self, D::Error> where D: serde::Deserializer<'de> {
                serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct Visitor<'de> {
            lifetime: serde::export::PhantomData<&'de ()>,
        }
        impl <'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Credentials<AnyPermissions>;
            fn expecting(&self, formatter: &mut serde::export::Formatter) -> serde::export::fmt::Result {
                serde::export::Formatter::write_str(formatter, "struct Credentials")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de> {
                let permissions = serde::de::SeqAccess::next_element::<Vec<Permission>>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(0usize, &"struct Credentials with 2 elements")
                )?;
                let user_id = serde::de::SeqAccess::next_element::<uuid::Uuid>(&mut seq)?.ok_or(
                    serde::de::Error::invalid_length(1usize, &"struct Credentials with 2 elements")
                )?;
                Ok(
                    Credentials{
                        level: PhantomData,
                        permissions: permissions,
                        user_id: user_id,
                    }
                )
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> serde::export::Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
                let mut permissions = None;
                let mut user_id = None;
                while let Some(key) = serde::de::MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Permissions => permissions = if permissions.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("permissions"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<Vec<Permission>>(&mut map)?)
                        },
                        Field::UserId => user_id = if user_id.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field("user_id"))
                        } else {
                            Some(serde::de::MapAccess::next_value::<uuid::Uuid>(&mut map)?)
                        },
                        _ => { let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(&mut map)?; },
                    }
                }
                let permissions = match permissions {
                    serde::export::Some(permissions) => permissions,
                    serde::export::None => serde::private::de::missing_field("permissions")?,
                };
                let user_id = match user_id {
                    serde::export::Some(user_id) => user_id,
                    serde::export::None => serde::private::de::missing_field("user_id")?,
                };
                serde::export::Ok(Credentials{
                    level: PhantomData,
                    permissions: permissions,
                    user_id: user_id,
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["permissions", "user_id"];
        serde::Deserializer::deserialize_struct(
            deserializer,
            "Credentials",
            FIELDS,
            Visitor {
                lifetime: serde::export::PhantomData,
            },
        )
    }
}
impl<L> Credentials<L> {
    const AUTH_COOKIE_NAME: &'static str = "_";
    pub fn to_user_id(self) -> uuid::Uuid {
        self.user_id
    }
    pub fn has_permissions(&self, req_perms: &[Permission]) -> bool {
        for req_perm in req_perms.iter() {
            if !self.permissions.contains(req_perm) {
                return false;
            }
        }
        true
    }
}
impl<L: Verifiable> Credentials<L> {
    /// Extracts an arbitrary credential from a provided token.
    fn extract(cookies: &Cookies, key: &CurrAndLastKey<paseto::v2::local::Algo>) -> Result<Credentials<AnyPermissions>, ()> {
        let auth_cookie = cookies.get(Self::AUTH_COOKIE_NAME).ok_or(())?;
        let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());

        let token: paseto::token::Data<Credentials<AnyPermissions>, ()> = match paseto::v2::local::Protocol::decrypt(token, &key.curr) {
            Ok(dec) => dec,
            Err(_) => {
                let token = paseto::token::Packed::new(auth_cookie.value().as_bytes().to_vec());
                paseto::v2::local::Protocol::decrypt(token, &key.last).map_err(|_| ())?
            },
        };
        Ok(token.msg)
    }
    /// Creates a new Credentials object from a set of permissions and validates the permissions
    /// level requested.
    pub fn new(user_id: uuid::Uuid, permissions: Vec<Permission>) -> Option<Self> {
        if L::verify_slice(permissions.as_slice()) {
            Some(Self {
                level: PhantomData,
                user_id: user_id,
                permissions: permissions,
            })
        } else {
            None
        }
    }
}
impl<'a, 'r, L: Verifiable> FromRequest<'a, 'r> for Credentials<L> {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let key_store = req
            .guard::<State<Arc<KeyStore<paseto::v2::local::Algo>>>>()
            .map_failure(|_| (Status::InternalServerError, ()))?;
        let key_read_guard = match key_store.curr_and_last() {
            Ok(rg) => rg,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        };

        let cr = match Self::extract(&cookies, &*key_read_guard) {
            Ok(cr) => cr,
            Err(_) => return Outcome::Failure((Status::Forbidden, ())),
        };

        if L::verify(&cr) {
            Outcome::Success(Credentials {
                level: PhantomData,
                permissions: cr.permissions,
                user_id: cr.user_id,
            })
        } else {
            Outcome::Failure((Status::Forbidden, ()))
        }
    }
}
pub type CredentialToken = paseto::token::Data<Credentials<AnyPermissions>, ()>;

/// Password authentication data. Separated from AuthenticationData to allow for impl blocks. Will
/// go away once enum variants become types.
#[derive(Serialize, Deserialize)]
pub struct PasswordData {
    user_name: String,
    password: String,
}

/// Struct holding data needed for user creation.
struct UserCreationData {
    basic_info: users::NewNoMeta,
    privileged_info: Option<PrivilegedUserCreationData>,
}
impl UserCreationData {
    fn creator_id(&self) -> Option<uuid::Uuid> {
        self.privileged_info.as_ref()
            .map(|info| info.creating_user.user_id)
    }
    fn create(self, db: &db::DB) -> Result<(users::Data, Vec<permissions::Data>), AuthenticationError> {
        let created_user = db.create_user(users::New::from((
            &self.basic_info,
            self.creator_id(),
        )))?;
        let permissions = self.privileged_info.map(|p| p.create(db, &created_user)).transpose()?;
        Ok((created_user, permissions.unwrap_or(vec![])))
    }
}

/// Struct holding data needed for privileged user creation.
struct PrivilegedUserCreationData {
    creating_user: Credentials<AnyPermissions>,
    target_permissions: Vec<Permission>,
}
impl PrivilegedUserCreationData {
    fn create(self, db: &db::DB, target_user: &users::Data) -> Result<Vec<permissions::Data>, AuthenticationError> {
        // TODO create permission entries
        let target_permissions = &self.target_permissions;
        if target_permissions.is_empty() && (!self.creating_user.has_permissions(&[Permission::GrantPermission]) || !self.creating_user.has_permissions(target_permissions.as_slice())) {
            return Err(AuthenticationError::LackingPermissions);
        }
        Ok(db.create_all_permissions(target_permissions.iter().map(|target_permission| permissions::New {
            created_by: self.creating_user.user_id,
            user_id: target_user.id,
            permission: target_permission.as_str(),
        }).collect())?)
    }
}

/// Errors for authentication.
enum AuthenticationError {
    /// Database errored when attempting operation.
    Diesel(diesel::result::Error),
    /// Authenticated user lacks permissions to create a user.
    LackingPermissions,
    /// Credentials do not match user.
    BadCredentials,
    /// KeyStore is poisoned.
    KeyStorePoisoned,
}
impl From<diesel::result::Error> for AuthenticationError {
    fn from(e: diesel::result::Error) -> Self {
        Self::Diesel(e)
    }
}
impl From<AuthenticationError> for status::Custom<()> {
    fn from(e: AuthenticationError) -> Self {
        match e {
            AuthenticationError::Diesel(_) => status::Custom(Status::InternalServerError, ()),
            AuthenticationError::LackingPermissions => status::Custom(Status::InternalServerError, ()),
            AuthenticationError::BadCredentials => status::Custom(Status::InternalServerError, ()),
            AuthenticationError::KeyStorePoisoned => status::Custom(Status::InternalServerError, ()),
        }
    }
}

/// Encodes a pairing of input and stored credentials of same type.
pub enum AuthnWithStored<'a> {
    Password(&'a PasswordData, credentials::pw::Data),
}
impl<'a> AuthnWithStored<'a> {
    fn verify_with_err(self, key: &<ARGON2D as A>::Key) -> Result<(), ()> {
        match self {
            Self::Password(pw, hash_and_salt) => if <ARGON2D as HashA>::verify(
                &ARGON2D_MSG::new(
                    base64::decode(pw.password.as_bytes()).map_err(|_| ())?.clone(),
                    {
                        let mut buffer = [0; ARGON2D::SALT_LEN as usize];
                        buffer.copy_from_slice(hash_and_salt.salt.as_bytes());
                        Some(buffer)
                    },
                    Some(hash_and_salt.hash.len() as u32),
                ).map_err(|_| ())?,
                hash_and_salt.hash.as_bytes(),
                key,
            ) {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

/// Actual data that needs to be verified before someone can log in.
/// Currently only allows for passwords, but planning to support SSO and FIDO.
#[derive(Serialize, Deserialize)]
pub enum AuthenticationData {
    /// Data needed to fully specify a password credential from the request.
    Password(PasswordData),
}
impl AuthenticationData {
    /// Authenticates or creates user or modifies table.
    fn authenticate_or_create(
        &self,
        db: &db::DB,
        pw_key_store: &KeyStore<ARGON2D>,
        credentials: Option<&Credentials<AnyPermissions>>,
        user_to_create: Option<UserCreationData>,
    ) -> Result<(users::Data, Vec<Permission>), AuthenticationError> {
        let creator_id = user_to_create.as_ref().and_then(|c| c.creator_id());
        let (user, permissions) = match user_to_create {
            // need to create user
            Some(user_to_create) => user_to_create.create(db)?,
            // existing user
            None => self.find_user_with_permissions(db)?,
        };
        let key = &pw_key_store.curr_and_last().map_err(|_| AuthenticationError::KeyStorePoisoned)?.curr;
        let targeted_credential = match self.pair_with_stored(db, &user) {
            Err(diesel::result::Error::NotFound) => match credentials {
                // Need to create credential, iff one of the following:
                Some(c) => if c.user_id == user.id || CanEditUser::verify(c) {
                    // a) logged in
                    // b) has proper credentials
                    self.create(db, &user, c.user_id, &key)?
                } else {
                    return Err(AuthenticationError::LackingPermissions);
                },
                None => if let Some(creator_id) = creator_id {
                    // c) creating user
                    self.create(db, &user, creator_id, &key)?
                } else {
                    return Err(AuthenticationError::LackingPermissions)
                },
            },
            // Regular `?` behavior.
            result => result?,
        };
        targeted_credential.verify_with_err(&key)
            .map(|_| (user, permissions.iter().map(|p| Permission::from(p)).collect()))
            .map_err(|_| AuthenticationError::BadCredentials)
    }
    fn find_user_with_permissions(&self, db: &db::DB) -> Result<(users::Data, Vec<permissions::Data>), diesel::result::Error> {
        let user = match self {
            Self::Password(p) => db.find_user_by_user_name(p.user_name.as_str()),
        }?;
        let permissions = db.get_user_permissions(&user)?;
        Ok((user, permissions))
    }
    fn pair_with_stored(&self, db: &db::DB, user: &users::Data) -> Result<AuthnWithStored, diesel::result::Error> {
        match self {
            Self::Password(p) => db.find_pw_hash_by_user(user).map(move |d| AuthnWithStored::Password(p, d)),
        }
    }
    fn create(&self, db: &db::DB, user: &users::Data, creator_id: uuid::Uuid, key: &<ARGON2D as A>::Key) -> Result<AuthnWithStored, AuthenticationError> {
        Ok(match self {
            Self::Password(p) => {
                let msg = &ARGON2D_MSG::new_default_hash_len(
                    p.password.as_bytes().to_vec(),
                    None,
                );
                let generated_salt = msg.salt();
                let pw_hash = <ARGON2D as HashA>::sign(
                    msg,
                    key,
                );
                AuthnWithStored::Password(p, db.create_pw_hash(credentials::pw::New {
                    created_by: creator_id,
                    updated_by: creator_id,
                    user_id: user.id,
                    hash: base64::encode(pw_hash.as_slice()).as_str(),
                    salt: base64::encode(generated_salt).as_str(),
                })?)
            },
        })
    }
}

/// Route handler for the log in page.
#[get("/login")]
pub fn get() -> &'static str {
    "a login screen, eventually"
}

/// Message sent when attempting to log into the blog or when attempting to register for an account.
#[derive(Serialize, Deserialize)]
struct UserCreationDataNoCredentials {
    user_to_create: users::NewNoMeta,
    wanted_permissions: Option<Vec<Permission>>,
}
impl UserCreationDataNoCredentials {
    fn attach_credentials(self, credentials: Option<Credentials<AnyPermissions>>) -> Result<UserCreationData, ()> {
        Ok(UserCreationData {
            basic_info: self.user_to_create,
            privileged_info: self.wanted_permissions.map(|perms| Ok(PrivilegedUserCreationData {
                creating_user: credentials.ok_or(())?,
                target_permissions: perms,
            })).transpose()?,
        })
    }
}
/// Message sent when attempting to log into the blog or when attempting to register for an account.
#[derive(Serialize, Deserialize)]
struct LoginDataNoCredentials {
    login: AuthenticationData,
    user_to_create: Option<UserCreationDataNoCredentials>,
}
impl LoginDataNoCredentials {
    fn attach_credentials(self, credentials: Option<Credentials<AnyPermissions>>) -> Result<LoginData, ()> {
        Ok(LoginData {
            login: self.login,
            user_to_create: self.user_to_create
                .map(|c| c.attach_credentials(credentials))
                .transpose()?,
        })
    }
}
/// Message sent when attempting to log into the blog or when attempting to register for an account.
pub struct LoginData {
    login: AuthenticationData,
    user_to_create: Option<UserCreationData>,
}
impl FromDataSimple for LoginData {
    type Error = ();
    fn from_data(req: &Request, data: Data) -> OutcomeWithData<Self, Self::Error> {
        let credentials = match req.guard::<Option<Credentials<AnyPermissions>>>() {
            Outcome::Failure(f) => return OutcomeWithData::Failure((Status::InternalServerError, ())),
            Outcome::Forward(_) => return OutcomeWithData::Forward(data),
            Outcome::Success(cr) => cr,
        };

        let json = ContentType::JSON;
        if req.content_type() != Some(&ContentType::JSON) {
            return OutcomeWithData::Forward(data);
        }
        let size_limit = req.limits().get("json").unwrap_or(1 << 20);
        let mut to_create = String::with_capacity(512);
        let to_create = match data.open().take(size_limit).read_to_string(&mut to_create) {
            Ok(_) => to_create,
            Err(_) => return OutcomeWithData::Failure((Status::BadRequest, ())),
        };
        let to_create: LoginDataNoCredentials = match serde_json::from_str(to_create.as_str()) {
            Ok(to_create) => to_create,
            Err(_) => return OutcomeWithData::Failure((Status::BadRequest, ())),
        };

        match {
            to_create.user_to_create
                .map(|to_create| to_create.attach_credentials(credentials))
                .transpose()
        } {
            Ok(user_to_create) => OutcomeWithData::Success(Self {
                login: to_create.login,
                user_to_create: user_to_create,
            }),
            Err(_) => OutcomeWithData::Failure((Status::BadRequest, ())),
        }
    }
}

#[must_use]
fn add_authz_tok_if_absent(cookies: &mut Cookies, tok: CredentialToken, key: &<paseto::v2::local::Algo as A>::Key) -> Result<(), ()> {
    if cookies.get(Credentials::<()>::AUTH_COOKIE_NAME).is_some() {
        return Err(())
    };
    cookies.add(Cookie::new(
        Credentials::<()>::AUTH_COOKIE_NAME,
        paseto::v2::local::Protocol.encrypt(tok, key)
            .map_err(|_| ())
            .and_then(|s| str::from_utf8(&s).map(|s| s.to_owned()).map_err(|_| ()))?,
    ));
    Ok(())
}
/// Route handler for logging into the blog as well as creating an account.
#[post("/login", format = "json", data = "<data>")]
pub fn post(
    db: db::DB,
    mut cookies: Cookies,
    tok_key_store: State<Arc<KeyStore<paseto::v2::local::Algo>>>,
    pw_key_store: State<Arc<KeyStore<ARGON2D>>>,
    credentials: Option<Credentials<AnyPermissions>>,
    data: LoginData,
) -> Result<Redirect, status::Custom<()>> {
    let (user, perms) = data.login.authenticate_or_create(
        &db,
        &pw_key_store,
        credentials.as_ref(),
        data.user_to_create,
    ).map_err(|e| status::Custom::from(e))?;
    add_authz_tok_if_absent(
        &mut cookies,
        paseto::token::Data {
            msg: UnverifiedPermissionsCredential::new(user.id, perms).0,
            footer: None
        },
        &tok_key_store.curr_and_last().map_err(|_| status::Custom(Status::InternalServerError, ()))?.curr,
    ).map_err(|_| status::Custom(Status::InternalServerError, ()))?;
    Ok(Redirect::to("/")) // TODO create a landing page + replace
}
/// change a credential
#[patch("/login", format = "json", data = "<login_data>")]
pub fn patch(
    db: db::DB,
    login_data: Json<AuthenticationData>,
    cookies: Cookies,
    credentials: Option<Credentials<AnyPermissions>>,
) -> status::Custom<()> {
    // db.insert_user(basic);
    if credentials.is_some() {
        // create_new_and_replace(db, password, user);
    } else {
    }
    status::Custom(Status::new(501, "Not yet implemented"), ())
}
/// deletes an account
#[delete("/login")]
pub fn delete(db: db::DB) -> status::Custom<()> {
    // db.delete_user(db::find_user_by_hash()?);
    status::Custom(Status::new(501, "Not yet implemented"), ())
}

#[cfg(test)]
mod unit_tests {
}

