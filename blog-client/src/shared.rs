pub mod views;
pub mod retry;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Authorization {
    LoggedIn,
    LoggedOut,
}
