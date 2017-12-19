-- Your SQL goes here
CREATE TABLE users (
    id serial primary key,
    email varchar not null,
    password varchar not null,
    is_active boolean not null default 't'
)
