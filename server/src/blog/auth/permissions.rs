use std::str;
use serde::{
    Serialize,
    Deserialize,
};

use blog_db::models::*;

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
    EditUser,
    EditUserCredentials,
    GrantPermission,
    ViewPermission,
    DeletePermission,
    Custom { name: String },
}
// TODO make string const and lift into enum declaration
impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Self::EditPost => "edit_post",
            Self::CreatePost => "create_post",
            Self::DeletePost => "delete_post",
            Self::PublishPost => "publish_post",
            Self::ArchivePost => "archive_post",
            Self::CreateUser => "create_user",
            Self::EditUser => "edit_user",
            Self::EditUserCredentials => "edit_user_credentials",
            Self::GrantPermission => "grant_permission",
            Self::ViewPermission => "view_permission",
            Self::DeletePermission => "delete_permission",
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
            "edit_user" => Self::EditUser,
            "edit_user_credentials" => Self::EditUserCredentials,
            "grant_permission" => Self::GrantPermission,
            "view_permission" => Self::ViewPermission,
            "delete_permission" => Self::DeletePermission,
            p @ _ => Self::Custom { name: p.to_owned() },
        }
    }
}

/// Used to indicate that a type represents a permissions level.
pub trait Verifiable {
    const REQUIRED_PERMS: &'static[Permission];
    /// Verifies that the list of permissions passed in satisfies the constraints of the permission level.
    fn verify<T>(cred: &super::Credentials<T>) -> bool {
        cred.has_permissions(Self::REQUIRED_PERMS)
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

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to view permissions.
pub struct CanViewPermission;
impl Verifiable for CanViewPermission {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::ViewPermission];
}

/// Type to represent the Admin privlege level. This level of privlege represents at least the
/// right to delete permissions.
pub struct CanDeletePermission;
impl Verifiable for CanDeletePermission {
    const REQUIRED_PERMS: &'static[Permission] = &[Permission::DeletePermission];
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
        Permission::ViewPermission,
        Permission::DeletePermission,
    ];
}

/// Type to allow for the verification of a Credentials allowing for arbitrary permissions. Simply
/// a rename of the Unit type to make purpose clearer.
pub type Any = ();
impl Verifiable for Any {
    const REQUIRED_PERMS: &'static[Permission] = &[];
    fn verify<T>(_: &super::Credentials<T>) -> bool {
        true
    }
}

