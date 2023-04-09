-- Your SQL goes here
CREATE TABLE luckymon_history (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    user_id BIGINT,
    date_obtained DATE,
    pokemon_id BIGINT,
    shiny BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (id)
)

