CREATE TABLE share_links (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	deck_id    uuid NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
	token_hash text NOT NULL UNIQUE,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	revoked_at timestamptz
);
CREATE UNIQUE INDEX share_links_live_idx ON share_links (deck_id) WHERE revoked_at IS NULL;

CREATE TABLE exports (
	id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	deck_id     uuid NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
	rev         bigint NOT NULL,
	format      text NOT NULL CHECK (format IN ('html', 'pdf', 'pptx', 'mp4')),
	status      text NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
	error       text NOT NULL DEFAULT '',
	file_name   text NOT NULL DEFAULT '',
	size        bigint NOT NULL DEFAULT 0,
	created_by  bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at  timestamptz NOT NULL DEFAULT now(),
	finished_at timestamptz
);
CREATE INDEX exports_created_idx ON exports (created_at);
