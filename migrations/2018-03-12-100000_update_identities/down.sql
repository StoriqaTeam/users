-- This file should undo anything in `up.sql`
UPDATE identities set provider = '' where email = 'admin@storiqa.com';