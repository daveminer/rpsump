// @generated automatically by Diesel CLI.

diesel::table! {
    refresh_token (id) {
        id -> Integer,
        user_id -> Integer,
        token -> Text,
        expires_at -> Timestamp,
        revoked_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    garden_schedule (id) {
        id -> Integer,
        name -> Text,
        active -> Bool,
        start_times -> Text,
        days_of_week -> Text,
        duration_secs -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        skip_on_rain -> Bool,
    }
}

diesel::table! {
    garden_event (id) {
        id -> Integer,
        schedule_id -> Nullable<Integer>,
        source -> Text,
        status -> Text,
        scheduled_for -> Timestamp,
        duration_secs -> Integer,
        start_time -> Nullable<Timestamp>,
        end_time -> Nullable<Timestamp>,
        created_at -> Timestamp,
        schedule_name -> Nullable<Text>,
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

diesel::joinable!(garden_event -> garden_schedule (schedule_id));
diesel::joinable!(refresh_token -> user (user_id));
diesel::joinable!(user_event -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    garden_event,
    garden_schedule,
    refresh_token,
    sump_event,
    user,
    user_event,
);
