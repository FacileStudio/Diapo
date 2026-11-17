import type { Awareness } from 'y-protocols/awareness';
import type { Peer } from './types';

/** This person as the room shows them: the `user` field encre's carets read too. */
export type DeckUser = { name: string; color?: string };

const COLORS = ['#e5484d', '#f76b15', '#ffc53d', '#46a758', '#12a594', '#0090ff', '#6e56cf', '#d6409f'];

/** A stable colour for a Yjs client id. */
export function colorOf(client: number): string {
	return COLORS[Math.abs(client) % COLORS.length];
}

/** Publishes this client's name, colour, slide and selection on the room awareness. */
export function publish(awareness: Awareness, user: DeckUser, slide: string | null, selection: string[]): void {
	const color = user.color ?? colorOf(awareness.clientID);
	const mine = awareness.getLocalState() ?? {};
	const sameUser = mine.user?.name === user.name && mine.user?.color === color;
	if (!sameUser) awareness.setLocalStateField('user', { name: user.name, color });
	awareness.setLocalStateField('deck', { slide, selection });
}

function peerOf(client: number, state: Record<string, unknown>): Peer | null {
	const user = state.user as { name?: unknown; color?: unknown } | undefined;
	const deck = state.deck as { slide?: unknown; selection?: unknown } | undefined;
	if (!user && !deck) return null;
	const selection = Array.isArray(deck?.selection) ? deck.selection.filter((id) => typeof id === 'string') : [];
	return {
		client,
		name: typeof user?.name === 'string' ? user.name : '',
		color: typeof user?.color === 'string' ? user.color : colorOf(client),
		slide: typeof deck?.slide === 'string' ? deck.slide : null,
		selection
	};
}

/** Everyone else in the room, by client id. */
export function readPeers(awareness: Awareness): Peer[] {
	const out: Peer[] = [];
	awareness.getStates().forEach((state, client) => {
		const peer = client === awareness.clientID ? null : peerOf(client, state);
		if (peer) out.push(peer);
	});
	return out.sort((a, b) => a.client - b.client);
}
