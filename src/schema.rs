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
    luckymon_battle (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        guild_id -> Nullable<Int8>,
        battle_type -> Nullable<Varchar>,
        challenger_id -> Nullable<Int8>,
        opponent_id -> Nullable<Int8>,
        opponent_name -> Nullable<Varchar>,
        battlefield -> Nullable<Varchar>,
        seed -> Nullable<Int8>,
        winner_id -> Nullable<Int8>,
        turn_log -> Nullable<Text>,
        counted -> Nullable<Bool>,
        finished_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    luckymon_battle_team (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        battle_id -> Uuid,
        user_id -> Nullable<Int8>,
        slot -> Nullable<Int4>,
        pokemon_id -> Nullable<Int8>,
        shiny -> Nullable<Bool>,
    }
}

diesel::table! {
    luckymon_team (id) {
        id -> Uuid,
        last_modified_date -> Timestamp,
        user_id -> Nullable<Int8>,
        slot -> Nullable<Int4>,
        pokemon_id -> Nullable<Int8>,
        shiny -> Nullable<Bool>,
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

diesel::table! {
    pokemon_stat (pokemon_id) {
        pokemon_id -> Int8,
        name -> Varchar,
        hp -> Int4,
        atk -> Int4,
        def -> Int4,
        spa -> Int4,
        spd -> Int4,
        spe -> Int4,
        type_1 -> Varchar,
        type_2 -> Nullable<Varchar>,
        bst -> Int4,
    }
}

diesel::joinable!(luckymon_battle_team -> luckymon_battle (battle_id));

diesel::allow_tables_to_appear_in_same_query!(
    event,
    faq,
    luckymon_battle,
    luckymon_battle_team,
    luckymon_history,
    luckymon_team,
    pin,
    pokemon_stat,
);
