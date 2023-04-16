// @generated automatically by Diesel CLI.

diesel::table! {
    sump_events (id) {
        id -> Integer,
        kind -> Text,
        info -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}
