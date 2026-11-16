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
