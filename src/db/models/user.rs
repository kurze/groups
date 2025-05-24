use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[native_model(id = 1, version = 2, from = V1)]
#[native_db]
pub struct V2 {
    #[primary_key]
    pub id: u32,
    #[secondary_key(unique)]
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl V2 {
    pub fn new(email: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: rand::random::<u32>(),
            email,
            name: String::new(),
            password_hash: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl From<V1> for V2 {
    fn from(p: V1) -> Self {
        Self {
            id: p.id,
            email: p.email,
            name: p.name,
            password_hash: None,
            created_at: p.created_at,
            updated_at: p.created_at, // Use created_at as initial updated_at
            deleted_at: p.deleted_at,
        }
    }
}

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

impl From<V2> for V1 {
    fn from(p: V2) -> Self {
        Self {
            id: p.id,
            email: p.email,
            name: p.name,
            created_at: p.created_at,
            deleted_at: p.deleted_at,
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
