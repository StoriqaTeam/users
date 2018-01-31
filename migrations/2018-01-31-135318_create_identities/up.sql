-- Your SQL goes here
CREATE TABLE identities (
    user_id INTEGER UNIQUE REFERENCES users ON DELETE CASCADE,
    email VARCHAR NOT NULL,
    password VARCHAR NULL,
    provider VARCHAR
);

CREATE UNIQUE INDEX identities_user_id_idx ON identities (user_id);