DROP INDEX IF EXISTS users_reset_tokens_uuid_idx;

ALTER TABLE reset_tokens DROP COLUMN uuid;
