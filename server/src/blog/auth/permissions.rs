//! A collection of classes used to represent and verify permissions in the
//! [`Credentials`](crate::blog::auth::Credentials) struct.

use serde::{Deserialize, Serialize};
use std::str;

use blog_db::models::*;

/// Used to indicate that a type represents a permissions level. A Custom variant is listed for
/// anything not covered by the enum.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum Permission {
    /// Permission allowing for editing of posts.
    EditPost,
    /// Permission allowing for creation of posts.
    CreatePost,
    /// Permission allowing for deletion of posts.
    DeletePost,
    /// Permission allowing for publishing of posts.
    PublishPost,
    /// Permission allowing for archival of posts.
    ArchivePost,
    /// Permission allowing for creation of other users.
    CreateUser,
    /// Permission allowing for editing of other users.
    EditUser,
    /// Permission allowing for deletion of other users.
    DeleteUser,
    /// Permission allowing for editing/deletion of login credentials.
    EditUserCredentials,
    /// Permission to grant permissions to other users.
    GrantPermission,
    /// Permission to view permissions of other users.
    ViewPermission,
    /// Permission to delete permissions of other users.
    DeletePermission,
    /// Arbitrary permission, just in case.
    Custom { name: String },
}
// TODO make string const and lift into enum declaration when const generics.
impl Permission {
    /// Convert a [`Permission`](crate::blog::auth::permissions::Permission) to a [`&str`].
    pub fn as_str(&self) -> &str {
        match self {
            Self::EditPost => "edit_post",
            Self::CreatePost => "create_post",
            Self::DeletePost => "delete_post",
            Self::PublishPost => "publish_post",
            Self::ArchivePost => "archive_post",
            Self::CreateUser => "create_user",
            Self::EditUser => "edit_user",
            Self::DeleteUser => "delete_user",
            Self::EditUserCredentials => "edit_user_credentials",
            Self::GrantPermission => "grant_permission",
            Self::ViewPermission => "view_permission",
            Self::DeletePermission => "delete_permission",
            Self::Custom { name } => &name,
        }
    }
}
// TODO make string const and lift into enum declaration when const generics.
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
            "delete_user" => Self::DeleteUser,
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
    const REQUIRED_PERMS: &'static [Permission];
    /// Verifies that the credentials passed in satisfies the permission level.
    fn verify<T>(cred: &super::Credentials<T>) -> bool {
        Self::verify_slice(cred.permissions())
    }
    /// Verifies if a provided slice of permissions satisfies the permissions level.
    fn verify_slice(perms: &[Permission]) -> bool {
        Self::REQUIRED_PERMS
            .iter()
            .all(|req_perm| perms.contains(req_perm))
    }
}

/// This level of privlege represents at least the right to create user accounts.
pub struct CanCreateUser;
impl Verifiable for CanCreateUser {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::CreateUser];
}

/// This level of privlege represents at least the right to change user account information.
pub struct CanEditUser;
impl Verifiable for CanEditUser {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::EditUser];
}

/// This level of privlege represents at least the right to delete user accounts.
pub struct CanDeleteUser;
impl Verifiable for CanDeleteUser {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::DeleteUser];
}

/// This level of privlege represents at least the right to edit blog posts.
pub struct CanEditUserCredentials;
impl Verifiable for CanEditUserCredentials {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::EditUserCredentials];
}

/// This level of privlege represents at least the right to edit blog posts.
pub struct CanEdit;
impl Verifiable for CanEdit {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::EditPost];
}

/// This level of privlege represents at least the right to delete blog posts.
pub struct CanDelete;
impl Verifiable for CanDelete {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::DeletePost];
}

/// This level of privlege represents at least the right to create blog posts.
pub struct CanPost;
impl Verifiable for CanPost {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::CreatePost];
}

/// This level of privlege represents at least the right to publish blog posts.
pub struct CanPublish;
impl Verifiable for CanPublish {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::PublishPost];
}

/// This level of privlege represents at least the right to archive blog posts.
pub struct CanArchive;
impl Verifiable for CanArchive {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::ArchivePost];
}

/// This level of privlege represents at least the right to grant permissions.
///
/// NOTE: Can only grant permissions they already have.
pub struct CanGrantPermission;
impl Verifiable for CanGrantPermission {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::GrantPermission];
}

/// This level of privlege represents at least the right to view permissions.
pub struct CanViewPermission;
impl Verifiable for CanViewPermission {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::ViewPermission];
}

/// This level of privlege represents at least the right to delete permissions.
pub struct CanDeletePermission;
impl Verifiable for CanDeletePermission {
    const REQUIRED_PERMS: &'static [Permission] = &[Permission::DeletePermission];
}

/// Type to allow for the verification of a Credentials allowing for arbitrary permissions. Simply
/// a rename of the () type to make purpose clearer.
pub type Any = ();
impl Verifiable for Any {
    const REQUIRED_PERMS: &'static [Permission] = &[];
    fn verify<T>(_: &super::Credentials<T>) -> bool {
        true
    }
}
