// @generated automatically by Diesel CLI.

diesel::table! {
    event (id) {
        id -> Uuid,
        last_modified_date -> Nullable<Timestamp>,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        start_date -> Nullable<Date>,
        end_date -> Nullable<Date>,
    }
}

diesel::table! {
    faq (id) {
        id -> Uuid,
        last_modified_date -> Nullable<Timestamp>,
        question -> Nullable<Varchar>,
        answer -> Nullable<Varchar>,
    }
}

diesel::table! {
    pin (id) {
        id -> Uuid,
        last_modified_date -> Nullable<Timestamp>,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password -> Varchar,
        first_name -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    event,
    faq,
    pin,
    users,
);
