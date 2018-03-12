-- Your SQL goes here
DROP TRIGGER IF EXISTS set_updated_at on identities;
UPDATE identities set provider = 'Email' where email = 'admin@storiqa.com';
