UPDATE reset_tokens SET token_type = 'email_verify' where token_type = 'EmailVerify';
UPDATE reset_tokens SET token_type = 'password_reset' where token_type = 'PasswordReset';
UPDATE reset_tokens SET token_type = 'undefined' where token_type = 'Undefined';
