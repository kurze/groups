use once_cell::sync::Lazy;

mod group;
mod user;

pub type User = user::V1;
pub(super) type UserKey = user::V1Key;

pub type Group = group::V1;
pub(super) type GroupKey = group::V1Key;

pub static MODELS: Lazy<native_db::Models> = Lazy::new(|| {
    let mut models = native_db::Models::new();
    models.define::<user::V0>().unwrap();
    models.define::<user::V1>().unwrap();
    models.define::<group::V1>().unwrap();
    models
});
