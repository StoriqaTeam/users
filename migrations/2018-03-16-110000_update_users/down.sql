-- This file should undo anything in `up.sql`
UPDATE users set saga_id = '' where email = 'admin@storiqa.com';