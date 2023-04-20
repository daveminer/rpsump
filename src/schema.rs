// @generated automatically by Diesel CLI.

diesel::table! {
    sump_event (id) {
        id -> Integer,
        kind -> Text,
        info -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Nullable<Integer>,
        password_hash -> Text,
        email -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    sump_event,
    user,
);
