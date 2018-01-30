-- This file should undo anything in `up.sql`
CREATE TYPE IF NOT EXISTS provider_type AS ENUM ('email', 'unverified_email', 'facebook', 'google');
CREATE TYPE IF NOT EXISTS gender_type AS ENUM ('male', 'female', 'undefined');

ALTER TABLE users ALTER COLUMN gender SET NOT NULL;
ALTER TABLE identities ALTER COLUMN provider SET NOT NULL;

ALTER TABLE users ALTER COLUMN gender TYPE gender_type;
ALTER TABLE identities ALTER COLUMN provider TYPE provider_type;