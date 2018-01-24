-- This file should undo anything in `up.sql`
DROP TYPE IF EXISTS provider_type;
DROP TYPE IF EXISTS gender_type;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS identities;

