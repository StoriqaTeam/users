ALTER TABLE reset_tokens ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp;
