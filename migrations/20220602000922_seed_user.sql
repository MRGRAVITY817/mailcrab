-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
	'ddf8994f-d522-4659-8d02-c1d479057be6',
	'admin',
	'$argon2id$v=19$m=15000,t=2,p=1$r1cYrrV1uYUA1KgMSavF7w$9rHUHJ1JUeBUh4JeZvIu/ncj/axYX9lGAsvFqm+fq18'
);
