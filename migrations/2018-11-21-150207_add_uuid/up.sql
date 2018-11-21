ALTER TABLE reset_tokens ADD COLUMN uuid uuid;

UPDATE reset_tokens SET uuid = uuid_generate_v4();

CREATE UNIQUE INDEX IF NOT EXISTS users_reset_tokens_uuid_idx ON reset_tokens (uuid);

ALTER TABLE reset_tokens ALTER COLUMN uuid SET NOT NULL;
