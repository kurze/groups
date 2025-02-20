use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use data::{v1::UserKey, User};
use std::sync::Mutex;
use native_db::*;
use once_cell::sync::Lazy;


struct AppStateWithCounter {
    counter: Mutex<i32>,
}

#[get("/")]
async fn hello(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Request number: {counter}"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    test_native().unwrap();
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    // let mut db = establish_connection();
    // if let Err(e) = diesel::sql_query("SELECT 1").execute(&mut db) {
    //     eprintln!("Error checking database connection: {}", e);
    // } else {
    //     println!("Database connection established successfully.");
    // }

    HttpServer::new(move || App::new().app_data(counter.clone()).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

// pub fn establish_connection() -> SqliteConnection {
//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     SqliteConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }

pub mod data {
    use native_db::{native_db, ToKey};
    use native_model::{native_model, Model};
    use serde::{Deserialize, Serialize};

    pub type User = v1::User;
    pub type Group = v1::Group;

    pub mod v1 {
        use super::*;

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        pub struct User {
            #[primary_key]
            pub id: u32,
            #[secondary_key(unique)]
            pub email: String,
            pub name: String,
            pub created_at: chrono::NaiveDateTime,
            pub deleted_at: Option<chrono::NaiveDateTime>,
        }

        impl User {
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


        #[derive(Serialize, Deserialize, Debug)]
        #[native_model(id = 2, version = 1)]
        #[native_db]
        pub struct Group {
            #[primary_key]
            pub id: u32,
            pub name: String,
            pub created_at: chrono::NaiveDateTime,
            pub deleted_at: Option<chrono::NaiveDateTime>,
        }
    }
}

// Define the models
// The lifetime of the models needs to be longer or equal to the lifetime of the database.
// In many cases, it is simpler to use a static variable but it is not mandatory.
static MODELS: Lazy<Models> = Lazy::new(|| {
   let mut models = Models::new();
   // It's a good practice to define the models by specifying the version
   models.define::<data::v1::User>().unwrap();
   models
});



fn test_native() -> Result<(), db_type::Error> {
    // ... database creation see previous example
    let db = Builder::new().create_in_memory(&MODELS)?;

    // Insert a user
    let rw = db.rw_transaction()?;
    // It's a good practice to use the latest version in your application

    let mut u = User::new("alice@wonderla.nd".to_string());
    u.name = "Alice".to_string();
    let id = u.id;

    rw.insert(u)?;
    rw.commit()?;

    // Get the user
    let r = db.r_transaction()?;
    {
        let result: Result<Option<data::v1::User>, db_type::Error> = r.get().primary(id);
        assert!(result.is_ok());
        let user:data::User = result.unwrap().unwrap();
        assert_eq!(user.name, "Alice");
    }
    {
        let result: Result<Option<data::v1::User>, db_type::Error> = r.get().secondary(UserKey::email, "alice@wonderla.nd".to_string());
        if result.is_err() {
            eprintln!("Error retrieving user by email: {}", result.unwrap_err());
        } else {
            let user: data::User = result.unwrap().unwrap();
            assert_eq!(user.name, "Alice");
        }
        // assert!(result.is_ok());
        // let user:data::User = result.unwrap().unwrap();
        // assert_eq!(user.name, "Alice");
    }

    Ok(())
}