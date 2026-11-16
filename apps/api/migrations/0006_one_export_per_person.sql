UPDATE exports SET status = 'failed', error = 'the server restarted during this export; start it again',
	finished_at = now() WHERE status IN ('queued', 'running');
CREATE UNIQUE INDEX exports_one_live_idx ON exports (created_by) WHERE status IN ('queued', 'running');
