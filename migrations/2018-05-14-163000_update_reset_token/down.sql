DROP INDEX reset_tokens_email_token_type_idx;

ALTER TABLE reset_tokens DROP COLUMN token_type;

CREATE UNIQUE INDEX reset_tokens_email_idx ON reset_tokens (email);