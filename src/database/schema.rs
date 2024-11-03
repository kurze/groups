// @generated automatically by Diesel CLI.

diesel::table! {
    groups (id) {
        id -> Integer,
        name -> Text,
        creation_date -> Timestamp,
        deletion_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        emmail -> Text,
        password -> Text,
        creation_date -> Timestamp,
        deletion_date -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(groups, users,);
