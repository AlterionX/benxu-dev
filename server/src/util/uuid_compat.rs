pub fn ruuid_to_uuid(ruuid: rocket_contrib::uuid::Uuid) -> uuid::Uuid {
    let bytes = ruuid.into_inner().as_bytes().clone();
    uuid::Uuid::from_bytes(bytes)
}
