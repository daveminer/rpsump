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
        id -> Integer,
        email -> Text,
        email_verification_token -> Nullable<Text>,
        email_verification_token_expires_at -> Nullable<Timestamp>,
        email_verified_at -> Nullable<Timestamp>,
        password_hash -> Text,
        password_reset_token -> Nullable<Text>,
        password_reset_token_expires_at -> Nullable<Text>,
        activated -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    sump_event,
    user,
);
