// @generated automatically by Diesel CLI.

diesel::table! {
    groups (id) {
        id -> Integer,
        name -> Text,
        creation_date -> Timestamp,
        deletion_date -> Nullable<Timestamp>,
    }
}
