-- Your SQL goes here
CREATE TYPE provider_type AS ENUM ('email', 'unverified_email', 'facebook', 'google');
CREATE TYPE gender_type AS ENUM ('male', 'female', 'undefined');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT 'f',
    phone VARCHAR,
    phone_verified BOOLEAN NOT NULL DEFAULT 'f',
    is_active BOOLEAN NOT NULL DEFAULT 't',
    first_name VARCHAR,
    last_name VARCHAR,
    middle_name VARCHAR,
    gender gender_type NOT NULL,
    birthdate VARCHAR,
    last_login_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE identities (
    user_id INTEGER UNIQUE REFERENCES users ON DELETE CASCADE,
    user_email VARCHAR NOT NULL,
    user_password VARCHAR NULL,
    provider provider_type NOT NULL
);

CREATE UNIQUE INDEX identities_user_id_idx ON identities (user_id);
