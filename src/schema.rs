// @generated automatically by Diesel CLI.

diesel::table! {
    event (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        guild_id -> Nullable<Int8>,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    faq (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        guild_id -> Nullable<Int8>,
        question -> Nullable<Varchar>,
        answer -> Nullable<Varchar>,
    }
}

diesel::table! {
    pin (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        guild_id -> Nullable<Int8>,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    event,
    faq,
    pin,
);
