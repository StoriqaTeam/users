ALTER TABLE user_roles ADD COLUMN name VARCHAR NOT NULL DEFAULT 'user';
ALTER TABLE user_roles ADD COLUMN data JSONB;
UPDATE user_roles SET name = role;
ALTER TABLE user_roles DROP COLUMN role;
ALTER TABLE user_roles DROP COLUMN id;
ALTER TABLE user_roles ADD COLUMN id UUID PRIMARY KEY DEFAULT uuid_generate_v4();
