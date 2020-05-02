pub mod views;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoggedIn {
    LoggedIn,
    LoggedOut,
}