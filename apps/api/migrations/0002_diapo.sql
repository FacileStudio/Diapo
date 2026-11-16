CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE spaces (
	id         text PRIMARY KEY,
	name       text NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now(),
	deleted_at timestamptz
);

CREATE TABLE space_members (
	space_id text NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
	subject  text NOT NULL,
	email    text NOT NULL,
	name     text NOT NULL DEFAULT '',
	picture  text NOT NULL DEFAULT '',
	role     text NOT NULL CHECK (role IN ('member', 'admin', 'owner')),
	PRIMARY KEY (space_id, subject)
);
CREATE INDEX space_members_subject_idx ON space_members (subject);

CREATE TABLE directory_state (
	id      smallint PRIMARY KEY CHECK (id = 1),
	version bigint NOT NULL
);

CREATE TABLE deletion_state (
	id         smallint PRIMARY KEY CHECK (id = 1),
	state      bytea NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE decks (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	space_id   text REFERENCES spaces(id) ON DELETE CASCADE,
	owner_id   bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	title      text NOT NULL DEFAULT '',
	source     text NOT NULL,
	rev        bigint NOT NULL DEFAULT 1,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	updated_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	updated_at timestamptz NOT NULL DEFAULT now(),
	trashed_at timestamptz,
	CHECK (space_id IS NOT NULL OR owner_id IS NOT NULL)
);
CREATE INDEX decks_space_idx ON decks (space_id, updated_at DESC) WHERE trashed_at IS NULL;
CREATE INDEX decks_owner_idx ON decks (owner_id, updated_at DESC) WHERE space_id IS NULL AND trashed_at IS NULL;
CREATE INDEX decks_trash_idx ON decks (trashed_at) WHERE trashed_at IS NOT NULL;

CREATE TABLE deck_revisions (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	deck_id    uuid NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
	rev        bigint NOT NULL,
	source     text NOT NULL,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	origin     text NOT NULL CHECK (origin IN ('web', 'mcp', 'cli')),
	created_at timestamptz NOT NULL DEFAULT now(),
	UNIQUE (deck_id, rev)
);

CREATE TABLE assets (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	deck_id    uuid NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
	name       text NOT NULL,
	mime       text NOT NULL,
	size       bigint NOT NULL,
	sha256     text NOT NULL,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	UNIQUE (deck_id, name)
);
CREATE INDEX assets_sha256_idx ON assets (sha256);
