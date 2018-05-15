DROP INDEX reset_tokens_email_idx;

DELETE FROM reset_tokens;

ALTER TABLE reset_tokens ADD COLUMN token_type VARCHAR NOT NULL;

CREATE UNIQUE INDEX reset_tokens_email_token_type_idx ON reset_tokens (email, token_type);
