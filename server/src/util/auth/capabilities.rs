//! A collection of classes used to represent and verify capabilities in the
//! [`Capabilities`](crate::blog::auth::Capabilities) struct.

use serde::{Deserialize, Serialize};
use std::str;

use blog_db::models::*;

/// Used to indicate that a type represents a capabilities level. A Custom variant is listed for
/// anything not covered by the enum.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum Capability {
    /// Capability allowing for editing of posts.
    EditPost,
    /// Capability allowing for creation of posts.
    CreatePost,
    /// Capability allowing for deletion of posts.
    DeletePost,
    /// Capability allowing for publishing of posts.
    PublishPost,
    /// Capability allowing for archival of posts.
    ArchivePost,
    /// Capability allowing for creation of other users.
    CreateUser,
    /// Capability allowing for editing of other users.
    EditUser,
    /// Capability allowing for deletion of other users.
    DeleteUser,
    /// Capability allowing for editing/deletion of login credentials.
    EditUserCredentials,
    /// Capability to grant capabilities to other users.
    GrantCapability,
    /// Capability to view capabilities of other users.
    ViewCapability,
    /// Capability to delete capabilities of other users.
    DeleteCapability,
    /// Arbitrary capability, just in case.
    Custom { name: String },
}
// TODO make string const and lift into enum declaration when const generics.
impl Capability {
    /// Convert a [`Capability`](crate::blog::auth::capabilities::Capability) to a [`&str`].
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
            Self::GrantCapability => "grant_capability",
            Self::ViewCapability => "view_capability",
            Self::DeleteCapability => "delete_capability",
            Self::Custom { name } => &name,
        }
    }
}
// TODO make string const and lift into enum declaration when const generics.
impl From<&capabilities::Data> for Capability {
    fn from(perm: &capabilities::Data) -> Self {
        match perm.capability.as_str() {
            "edit_post" => Self::EditPost,
            "create_post" => Self::CreatePost,
            "delete_post" => Self::DeletePost,
            "publish_post" => Self::PublishPost,
            "archive_post" => Self::ArchivePost,
            "create_user" => Self::CreateUser,
            "edit_user" => Self::EditUser,
            "delete_user" => Self::DeleteUser,
            "edit_user_credentials" => Self::EditUserCredentials,
            "grant_capability" => Self::GrantCapability,
            "view_capability" => Self::ViewCapability,
            "delete_capability" => Self::DeleteCapability,
            p => Self::Custom { name: p.to_owned() },
        }
    }
}

/// Used to indicate that a type represents a capabilities level.
pub trait Verifiable {
    const REQUIRED_CAPS: &'static [Capability];
    /// Verifies that the credentials passed in satisfies the capabilities.
    fn verify<T>(cred: &super::Capabilities<T>) -> bool {
        Self::verify_slice(cred.capabilities())
    }
    /// Verifies if a provided slice of capabilities satisfies the capabilities level.
    fn verify_slice(caps: &[Capability]) -> bool {
        Self::REQUIRED_CAPS
            .iter()
            .all(|req_perm| caps.contains(req_perm))
    }
}

/// This level of privlege represents at least the right to create user accounts.
#[derive(Debug)]
pub struct CreateUser;
impl Verifiable for CreateUser {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::CreateUser];
}

/// This level of privlege represents at least the right to change user account information.
#[derive(Debug)]
pub struct EditUser;
impl Verifiable for EditUser {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::EditUser];
}

/// This level of privlege represents at least the right to delete user accounts.
#[derive(Debug)]
pub struct DeleteUser;
impl Verifiable for DeleteUser {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::DeleteUser];
}

/// This level of privlege represents at least the right to edit blog posts.
#[derive(Debug)]
pub struct EditUserCredentials;
impl Verifiable for EditUserCredentials {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::EditUserCredentials];
}

/// This level of privlege represents at least the right to edit blog posts.
#[derive(Debug)]
pub struct Edit;
impl Verifiable for Edit {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::EditPost];
}

/// This level of privlege represents at least the right to delete blog posts.
#[derive(Debug)]
pub struct Delete;
impl Verifiable for Delete {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::DeletePost];
}

/// This level of privlege represents at least the right to create blog posts.
#[derive(Debug)]
pub struct Post;
impl Verifiable for Post {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::CreatePost];
}

/// This level of privlege represents at least the right to publish blog posts.
#[derive(Debug)]
pub struct Publish;
impl Verifiable for Publish {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::PublishPost];
}

/// This level of privlege represents at least the right to archive blog posts.
#[derive(Debug)]
pub struct Archive;
impl Verifiable for Archive {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::ArchivePost];
}

/// This level of privlege represents at least the right to grant capabilities.
///
/// NOTE: Can only grant capabilities they already have.
#[derive(Debug)]
pub struct GrantCapability;
impl Verifiable for GrantCapability {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::GrantCapability];
}

/// This level of privlege represents at least the right to view capabilities.
#[derive(Debug)]
pub struct ViewCapability;
impl Verifiable for ViewCapability {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::ViewCapability];
}

/// This level of privlege represents at least the right to delete capabilities.
#[derive(Debug)]
pub struct DeleteCapability;
impl Verifiable for DeleteCapability {
    const REQUIRED_CAPS: &'static [Capability] = &[Capability::DeleteCapability];
}

/// Type to allow for the verification of a Capabilities allowing for arbitrary capabilities. Simply
/// a rename of the () type to make purpose clearer.
pub type Any = ();
impl Verifiable for Any {
    const REQUIRED_CAPS: &'static [Capability] = &[];
    fn verify<T>(_: &super::Capabilities<T>) -> bool {
        true
    }
}
