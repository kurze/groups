use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[native_model(id = 1, version = 1,from = V0)]
#[native_db]
pub struct V1 {
    #[primary_key]
    pub id: u32,
    #[secondary_key(unique)]
    pub email: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl V1 {
    pub fn new(email: String) -> Self {
        Self {
            id: rand::random::<u32>(),
            email,
            name: String::new(),
            created_at: chrono::Utc::now().naive_utc(),
            deleted_at: None,
        }
    }
}

impl From<V0> for V1 {
    fn from(p: V0) -> Self {
        Self {
            id: p.id,
            email: String::new(),
            name: String::new(),
            created_at: chrono::Utc::now().naive_utc(),
            deleted_at: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[native_model(id = 1, version = 0)]
#[native_db]
pub struct V0 {
    #[primary_key]
    pub id: u32,
}

impl From<V1> for V0 {
    fn from(p: V1) -> Self {
        Self { id: p.id }
    }
}
