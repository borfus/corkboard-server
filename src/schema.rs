// @generated automatically by Diesel CLI.

diesel::table! {
    event (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
        guild_id -> Nullable<Int8>,
    }
}

diesel::table! {
    faq (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        question -> Nullable<Varchar>,
        answer -> Nullable<Varchar>,
        guild_id -> Nullable<Int8>,
    }
}

diesel::table! {
    luckymon_history (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        user_id -> Nullable<Int8>,
        date_obtained -> Nullable<Date>,
        pokemon_id -> Nullable<Int8>,
        shiny -> Nullable<Bool>,
        pokemon_name -> Nullable<Varchar>,
        traded -> Nullable<Bool>,
    }
}

diesel::table! {
    pin (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        url -> Nullable<Varchar>,
        title -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        guild_id -> Nullable<Int8>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(event, faq, luckymon_history, pin,);
