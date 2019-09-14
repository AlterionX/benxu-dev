use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Query {
    user_id: Option<uuid::Uuid>,
    permission_ids: Option<Vec<uuid::Uuid>>,
}
impl Query {
    pub fn user_id(&self) -> Option<uuid::Uuid> {
        self.user_id
    }
    pub fn permission_ids(&self) -> Option<&[uuid::Uuid]> {
        self.permission_ids.as_ref().map(|pp| pp.as_slice())
    }
}

