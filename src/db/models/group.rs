use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct V1 {
    #[primary_key]
    pub id: u32,
    #[secondary_key]
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl V1 {
    pub fn new(name: String) -> Self {
        Self {
            id: rand::random::<u32>(),
            name,
            created_at: chrono::Utc::now().naive_utc(),
            deleted_at: None,
        }
    }
}
