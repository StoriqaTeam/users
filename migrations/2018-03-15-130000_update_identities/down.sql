-- This file should undo anything in `up.sql`
UPDATE identities set saga_id = '' where email = 'admin@storiqa.com';