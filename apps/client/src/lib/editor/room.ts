import { openRoom, socketUrl, type Connection, type Room, type Stop } from '@facile/salon';
import type * as Y from 'yjs';
import { API } from '$lib/backend';

export type { Connection, Room, Stop };

/** What the editor hears from its deck room. */
export type DeckRoomEvents = {
	onconnection: (state: Connection) => void;
	onstop: (stop: Stop, reason: string) => void;
	onsynced: () => void;
};

/** The y-websocket base URL and room name of deck `id` on this origin. */
export function deckRoomUrl(id: string, location: Pick<Location, 'protocol' | 'host'>): [string, string] {
	return [socketUrl('/api/decks', location), `${encodeURIComponent(id)}/ws`];
}

/** Opens the salon room of deck `id`: stop on 1008, 1009 and 1011, else reconnect with backoff. */
export function openDeckRoom(id: string, ydoc: Y.Doc, events: DeckRoomEvents): Room {
	const [base, room] = deckRoomUrl(id, window.location);
	return openRoom({ base, room }, ydoc, {
		onconnection: events.onconnection,
		onstop: events.onstop,
		onsynced: events.onsynced
	});
}

/** The URL the app loads a deck asset from, such as `assets/chart.png`. */
export function deckAssetUrl(id: string, name: string): string {
	return `${API}/decks/${encodeURIComponent(id)}/assets/${encodeURI(name)}`;
}
