mod db_setup;
use diesel::RunQueryDsl;
use groups::database::db::DB;
/*
#[test]
fn test_crud_users() {
    let conn = db_setup::create_test_db();
    let mut db = DB::new_with_connection(conn);

    // Clear the users table
    diesel::delete(groups::database::schema::users::dsl::users)
        .execute(&mut db.conn)
        .expect("Failed to clear users table");

    // Create a user
    let username = "test_user";
    let password = "password123";
    let user = db
        .create_user(username, password)
        .expect("Failed to create user");
    assert_eq!(user.username, username);
    assert_eq!(user.password, password);
    assert!(user.id > 0);
    assert!(user.deletion_date.is_none());

    // List users
    let users = db.list_users().expect("Failed to list users");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0], user);

    let username2 = "test_user2";
    let password2 = "password456";
    let user2 = db
        .create_user(username2, password2)
        .expect("Failed to create user");
    assert_eq!(user2.username, username2);
    assert_eq!(user2.password, password2);
    assert!(user2.id > 0);
    assert!(user2.deletion_date.is_none());

    let users = db.list_users().expect("Failed to list users");
    assert_eq!(users.len(), 2);
    assert!(users[0] != users[1]);
    assert!(users.contains(&user));
    assert!(users.contains(&user2));

    // Get user
    let user_from_db = db
        .get_user(user.id)
        .expect("Failed to get user")
        .expect("User not found");
    assert_eq!(user, user_from_db);

    // Update user
    let new_username = "updated_user";
    let new_password = "newpassword123";
    db.update_user(user.id, new_username, new_password)
        .expect("Failed to update user");
    let updated_user = db
        .get_user(user.id)
        .expect("Failed to get user")
        .expect("User not found");
    assert_eq!(updated_user.username, new_username);
    assert_eq!(updated_user.password, new_password);

    // Delete user
    db.delete_user(user.id).expect("Failed to delete user");
    let deleted_user = db.get_user(user.id).expect("Failed to get user");
    assert!(deleted_user.is_none());
}
*/