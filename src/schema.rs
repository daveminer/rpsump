// @generated automatically by Diesel CLI.

diesel::table! {
    irrigation_event (id) {
        id -> Integer,
        hose_id -> Integer,
        created_at -> Timestamp,
        end_time -> Nullable<Timestamp>,
        status -> Text,
        schedule_id -> Integer,
    }
}

diesel::table! {
    irrigation_schedule (id) {
        id -> Integer,
        name -> Text,
        start_time -> Timestamp,
        days_of_week -> Text,
        hoses -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

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
        email_verified_at -> Nullable<Timestamp>,
        password_hash -> Text,
        password_reset_token -> Nullable<Text>,
        password_reset_token_expires_at -> Nullable<Timestamp>,
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

diesel::joinable!(irrigation_event -> irrigation_schedule (schedule_id));
diesel::joinable!(user_event -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    irrigation_event,
    irrigation_schedule,
    sump_event,
    user,
    user_event,
);
