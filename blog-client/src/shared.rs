pub mod views;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Authorization {
    LoggedIn,
    LoggedOut,
}
