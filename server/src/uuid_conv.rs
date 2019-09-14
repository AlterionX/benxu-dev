pub trait FromRUuid {
    fn from_ruuid(ruuid: rocket_contrib::uuid::Uuid) -> Self;
}
impl FromRUuid for uuid::Uuid {
    fn from_ruuid(ruuid: rocket_contrib::uuid::Uuid) -> Self {
        uuid::Uuid::from_uuid_bytes(*ruuid.into_inner().as_bytes())
    }
}

