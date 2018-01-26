-- Your SQL goes here
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER UNIQUE REFERENCES users ON DELETE CASCADE,
    role_id SMALLINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX user_roles_user_id_idx ON user_roles (user_id);

SELECT diesel_manage_updated_at('user_roles');

INSERT INTO users (email, password) VALUES ('admin@storiqa.com', 'bqF5BkdsCS') ON CONFLICT (id) DO NOTHING;
INSERT INTO user_roles (user_id, role_id) SELECT id, 0 FROM users WHERE email = 'admin@storiqa.com' LIMIT 1;
