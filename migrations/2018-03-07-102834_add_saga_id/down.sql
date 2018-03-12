-- This file should undo anything in `up.sql`
ALTER TABLE users DROP COLUMN IF EXISTS saga_id;
ALTER TABLE identities DROP COLUMN IF EXISTS saga_id;