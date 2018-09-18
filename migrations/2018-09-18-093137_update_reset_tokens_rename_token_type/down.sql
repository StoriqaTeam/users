UPDATE reset_tokens SET token_type = 'EmailVerify' where token_type = 'email_verify';
UPDATE reset_tokens SET token_type = 'PasswordReset' where token_type = 'password_reset';
UPDATE reset_tokens SET token_type = 'Undefined' where token_type = 'undefined';
