ALTER TABLE exports
	ADD COLUMN steps  boolean NOT NULL DEFAULT false,
	ADD COLUMN done   integer NOT NULL DEFAULT 0,
	ADD COLUMN total  integer NOT NULL DEFAULT 0,
	ADD COLUMN notice text    NOT NULL DEFAULT '';
