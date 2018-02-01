-- Your SQL goes here
CREATE TABLE identities (
    user_id INTEGER UNIQUE REFERENCES users ON DELETE CASCADE,
    email VARCHAR NOT NULL CHECK (email = tolower(email)),
    password VARCHAR NULL,
    provider VARCHAR
);

CREATE UNIQUE INDEX identities_user_id_idx ON identities (user_id);
CREATE UNIQUE INDEX identities_email_idx ON identities (email);

SELECT diesel_manage_updated_at('identities');