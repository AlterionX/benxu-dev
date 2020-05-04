/// Used to mark structs that can be converted into a database record and saved or used to update a
/// preexisting row in the table.
pub trait SavableCredential {
    /// The object returned on succcess, typically the ORM's `Data` representation of the struct.
    type Success;
    /// The object returned on succcess, typically the ORM's `Error` type.
    type Error;
    /// Converts the credential and attempts to create a new row for the credential. Will return
    /// the created row on success.
    fn convert_and_save_with_capabilities(self) -> Result<Self::Success, Self::Error>;
    /// Converts the credential and attempts to update an existing row for the credential. Will
    /// return the updated row on success.
    fn convert_and_update_with_capabilities(self) -> Result<Self::Success, Self::Error>;
}
