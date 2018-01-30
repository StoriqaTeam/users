-- Your SQL goes here
ALTER TABLE users ALTER COLUMN gender TYPE VARCHAR;
ALTER TABLE identities ALTER COLUMN provider TYPE VARCHAR;

ALTER TABLE users ALTER COLUMN gender DROP NOT NULL;
ALTER TABLE identities ALTER COLUMN provider DROP NOT NULL;

DROP TYPE IF EXISTS provider_type;
DROP TYPE IF EXISTS gender_type;

-- email to lower case
UPDATE users SET email=lower(email);
UPDATE identities SET user_email=lower(user_email);