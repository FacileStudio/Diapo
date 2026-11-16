/** One row of `GET /api/decks`: a deck without its source. */
export interface DeckSummary {
	id: string;
	title: string;
	rev: number;
	updated_at: string;
}

/** A deck as `GET /api/decks/{id}` answers it. Every write names the `rev` it read. */
export interface Deck extends DeckSummary {
	source: string;
}

/** One problem the scene parser found in a source the API refused with 422. `line` counts from 1. */
export interface SceneError {
	line: number;
	message: string;
}

/** A file of a deck. `name` is the relative path the source writes, such as `assets/chart.png`. */
export interface Asset {
	name: string;
	mime: string;
	size: number;
	url: string;
}

/** The share link of a deck, as `GET` and `POST /api/decks/{id}/share` answer it. `url` comes once, on creation. */
export interface ShareLink {
	shared: boolean;
	created_at: string | null;
	url?: string;
}

/** A shared deck as `GET /api/public/decks/{token}` answers it. `frame` is the route of its sandboxed document. */
export interface PublicDeck {
	title: string;
	scenes: number;
	steps: number;
	width: number;
	height: number;
	rev: number;
	frame: string;
}

/** What a deck exports to. */
export type ExportFormat = 'html' | 'pptx' | 'pdf' | 'mp4';

/** One format of `GET /api/exports/formats`: whether an export of it would start now. */
export interface ExportCapability {
	format: ExportFormat;
	available: boolean;
	reason?: string;
	notes?: string;
}

/** An export job, as `POST /api/decks/{id}/exports` and `GET /api/exports/{id}` answer it. */
export interface ExportJob {
	id: string;
	format: ExportFormat;
	status: 'queued' | 'running' | 'done' | 'failed';
	error: string;
	file_name: string;
	size: number;
	download_url: string | null;
}

/** The person's Bulle assistant, as `GET /api/assistant` answers it. */
export interface AssistantProfile {
	name: string;
	accent: string | null;
}

/** An agent action waiting for the person's confirmation, as `GET /api/mcp/pending` lists it. `target` is the deck id. */
export interface PendingAction {
	id: string;
	kind: string;
	target: string;
	summary: string;
	client_id: string;
	created_at: string;
	expires_at: string;
	state: string;
}

/** The signed-in person, as `GET /api/auth/session` names them. */
export interface SessionUser {
	email: string;
	name?: string;
	avatar_url?: string;
}
