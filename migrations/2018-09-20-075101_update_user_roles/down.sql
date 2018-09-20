ALTER TABLE user_roles ADD COLUMN role VARCHAR NOT NULL DEFAULT 'user';
ALTER TABLE user_roles DROP COLUMN data;
UPDATE user_roles SET role = name;
ALTER TABLE user_roles DROP COLUMN name;
ALTER TABLE user_roles DROP COLUMN id;
CREATE SEQUENCE IF NOT EXISTS user_roles_id_seq;
ALTER TABLE user_roles ADD COLUMN id INTEGER PRIMARY KEY DEFAULT nextval('user_roles_id_seq');
