-- Luckybattle: 3v3 auto-resolved battles, PvP and PvE.
--
-- luckymon_history is deliberately untouched. Collections stay global
-- (your pokemon are yours in every guild); only the battle record carries
-- guild_id, which is what the leaderboards scope against.

CREATE TABLE luckymon_battle (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    guild_id BIGINT,
    battle_type VARCHAR,        -- 'pvp' | 'pve'
    challenger_id BIGINT,
    opponent_id BIGINT,         -- NULL for PvE
    opponent_name VARCHAR,      -- NPC trainer name, NULL for PvP
    battlefield VARCHAR,
    seed BIGINT,
    winner_id BIGINT,           -- NULL means the NPC won
    turn_log TEXT,              -- JSON string; see note below
    counted BOOLEAN DEFAULT TRUE,
    finished_at TIMESTAMP,
    PRIMARY KEY (id)
);

-- turn_log is TEXT rather than jsonb on purpose: diesel 1.4.8 only maps
-- the Jsonb sql type when built with its "serde_json" feature, which this
-- crate does not enable. The log is serialised and deserialised in Rust
-- either way, so a string column costs nothing.

CREATE TABLE luckymon_battle_team (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    battle_id uuid NOT NULL REFERENCES luckymon_battle(id),
    user_id BIGINT,             -- NULL for the NPC side of a PvE battle
    slot INTEGER,
    pokemon_id BIGINT,
    shiny BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (id)
);

-- A user's saved default team. Absent rows are fine: the server falls back
-- to auto-picking from the caller's untraded collection.
CREATE TABLE luckymon_team (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    user_id BIGINT,
    slot INTEGER,
    pokemon_id BIGINT,
    shiny BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX idx_luckymon_team_user_slot
    ON luckymon_team (user_id, slot);

CREATE INDEX idx_luckymon_battle_team_battle
    ON luckymon_battle_team (battle_id);

-- Drives the leaderboard: wins per user, per guild, per mode.
CREATE INDEX idx_luckymon_battle_leaderboard
    ON luckymon_battle (guild_id, battle_type, winner_id)
    WHERE counted = TRUE;

-- Drives the per-day anti-farming counts at battle creation time.
CREATE INDEX idx_luckymon_battle_challenger_day
    ON luckymon_battle (challenger_id, battle_type, last_modified_date);
