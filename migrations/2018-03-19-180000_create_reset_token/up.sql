-- Your SQL goes here
CREATE TABLE reset_tokens (
    token VARCHAR PRIMARY KEY,
    email VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('reset_tokens');
