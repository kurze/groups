use native_db::{self, Builder, db_type};

mod models;
pub mod user;

pub fn test_native() -> Result<(), db_type::Error> {
    // ... database creation see previous example
    let db = Builder::new().create_in_memory(&models::MODELS)?;

    // Insert a user
    let rw = db.rw_transaction()?;
    // It's a good practice to use the latest version in your application

    let mut u = models::User::new("alice@wonderla.nd".to_string());
    u.name = "Alice".to_string();
    let id = u.id;

    rw.insert(u)?;
    rw.commit()?;

    // Get the user
    let r = db.r_transaction()?;
    {
        let result: Result<Option<models::User>, db_type::Error> = r.get().primary(id);
        assert!(result.is_ok());
        let user: models::User = result.unwrap().unwrap();
        assert_eq!(user.name, "Alice");
    }
    {
        let result: Result<Option<models::User>, db_type::Error> = r
            .get()
            .secondary(models::UserKey::email, "alice@wonderla.nd".to_string());
        if result.is_err() {
            eprintln!("Error retrieving user by email: {}", result.unwrap_err());
        } else {
            let user: models::User = result.unwrap().unwrap();
            assert_eq!(user.name, "Alice");
        }
    }

    Ok(())
}
