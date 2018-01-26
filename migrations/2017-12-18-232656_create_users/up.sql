-- Your SQL goes here
CREATE TYPE provider_type AS ENUM ('email', 'unverified_email', 'facebook', 'google');
CREATE TYPE gender_type AS ENUM ('male', 'female', 'undefined');

CREATE TABLE users (
    id serial primary key,
    email varchar not null,
    email_verified boolean not null default 'f',
    phone varchar,
    phone_verified boolean not null default 'f',
    is_active boolean not null default 't',
    first_name varchar,
    last_name varchar,
    middle_name varchar,
    gender gender_type not null,
    birthdate varchar,
    last_login_at TIMESTAMP not null,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE identities (
    user_id integer UNIQUE REFERENCES users ON DELETE CASCADE,
    user_email varchar not null,
    user_password varchar null,
    provider provider_type not null
);

CREATE UNIQUE INDEX identities_user_id_idx ON identities (user_id);
