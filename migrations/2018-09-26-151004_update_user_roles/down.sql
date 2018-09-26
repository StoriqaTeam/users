ALTER TABLE user_roles DROP CONSTRAINT role;
ALTER TABLE user_roles ADD CONSTRAINT user_roles_user_id_key UNIQUE (user_id);
