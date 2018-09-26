ALTER TABLE user_roles DROP CONSTRAINT user_roles_user_id_key;
ALTER TABLE user_roles ADD CONSTRAINT role UNIQUE (user_id, name, data);
