UPDATE users
SET email_verified = false
WHERE id in (
	SELECT user_id
	FROM identities
	WHERE provider in ('google', 'facebook') );
