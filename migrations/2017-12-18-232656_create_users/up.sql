-- Your SQL goes here
CREATE TYPE provider_type AS ENUM ('email', 'unverified_email', 'facebook', 'google');
CREATE TYPE gender_type AS ENUM ('male', 'female', 'undefined');

CREATE TABLE users (
    id serial primary key,
    email varchar not null,
    email_verified boolean not null default 'f',
    phone varchar default null,
    phone_verified boolean not null default 'f',
    is_active boolean not null default 't',
    first_name varchar default null,
    last_name varchar default null,
    middle_name varchar default null,
    gender gender_type not null,
    birthdate varchar default null,
    last_login_at varchar not null,
    created_at varchar not null,
    updated_at varchar not null,
)

CREATE TABLE identities (
    user_id integer UNIQUE REFERENCES users ON DELETE CASCADE,
    user_email varchar not null,
    user_password varchar not null,
    provider provider_type not null,
)

CREATE UNIQUE INDEX identities_user_id_idx ON identities (user_id);
