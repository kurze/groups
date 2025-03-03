use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct V1 {
    #[primary_key]
    pub id: u32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
