-- Your SQL goes here
CREATE TABLE user_roles (
    id serial primary key,
    user_id integer UNIQUE REFERENCES users ON DELETE CASCADE,
    role_id smallint not null
);

CREATE UNIQUE INDEX user_roles_user_id_idx ON user_roles (user_id);

INSERT INTO users (email, password) VALUES ('admin@storiqa.com', 'bqF5BkdsCS');
INSERT INTO user_roles (user_id, role_id) SELECT id, 0 FROM users LIMIT 1;
