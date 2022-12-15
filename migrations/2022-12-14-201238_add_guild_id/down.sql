-- This file should undo anything in `up.sql`
ALTER TABLE event DROP COLUMN guild_id;
ALTER TABLE pin DROP COLUMN guild_id;
ALTER TABLE faq DROP COLUMN guild_id;

