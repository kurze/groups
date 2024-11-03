// use crate::test_db::create_test_db;
mod db_setup;
use groups::database::db::DB;

#[test]
fn test_crud_groups() {
    let conn = db_setup::create_test_db();
    let mut db = DB::new_with_connection(conn);

    // Clear existing groups
    db.clear_groups().expect("Failed to clear groups");

    // Create a group
    let group_name = "Test Group";
    let group = db.create_group(group_name).expect("Failed to create group");
    assert_eq!(group.name, group_name);
    println!("Group ID: {}", group.id);
    assert!(group.id > 0);
    assert!(group.deletion_date.is_none());

    // List groups
    let groups = db.list_groups().expect("Failed to list groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], group);

    let group_name = "Test Group 2";
    let group2 = db.create_group(group_name).expect("Failed to create group");
    assert_eq!(group2.name, group_name);
    assert!(group2.id > 0);
    assert!(group2.deletion_date.is_none());

    let groups = db.list_groups().expect("Failed to list groups");
    assert_eq!(groups.len(), 2);
    assert!(groups[0] != groups[1]);
    assert!(groups.contains(&group));
    assert!(groups.contains(&group2));

    // Get group
    let group_from_db = db
        .get_group(group.id)
        .expect("Failed to get group")
        .expect("Group not found");
    assert_eq!(group, group_from_db);

    // Update group
    let new_group_name = "Updated Test Group";
    db.update_group(group.id, new_group_name)
        .expect("Failed to update group");
    let updated_group = db
        .get_group(group.id)
        .expect("Failed to get group")
        .expect("Group not found");
    assert_eq!(updated_group.name, new_group_name);

    // Delete group
    db.delete_group(group.id).expect("Failed to delete group");
    let deleted_group = db.get_group(group.id).expect("Failed to get group");
    assert!(deleted_group.is_none());
}
