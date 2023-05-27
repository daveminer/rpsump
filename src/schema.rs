// @generated automatically by Diesel CLI.

diesel::table! {
    sump_event (id) {
        id -> Integer,
        kind -> Text,
        info -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        email -> Text,
        email_verification_token -> Nullable<Text>,
        email_verification_token_expires_at -> Nullable<Timestamp>,
        email_verified_at -> Nullable<Text>,
        password_hash -> Text,
        password_reset_token -> Nullable<Text>,
        password_reset_token_expires_at -> Nullable<Timestamp>,
        activated -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_event (id) {
        id -> Integer,
        user_id -> Integer,
        event_type -> Text,
        ip_address -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(user_event -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sump_event,
    user,
    user_event,
);
