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

INSERT INTO users (email, last_login_at) VALUES ('admin@storiqa.com', now()) ON CONFLICT (id) DO NOTHING;
INSERT INTO identities (user_id, email, password) SELECT id, email, 'bqF5BkdsCS' FROM users WHERE email = 'admin@storiqa.com' LIMIT 1;
INSERT INTO user_roles (user_id, role_id) SELECT id, 1 FROM users WHERE email = 'admin@storiqa.com' LIMIT 1;