import { backend } from '$lib/backend';
import type { Deck } from '$lib/types';

/** Loads one deck for a page: the deck, or `missing` when the API refuses it. A late answer for another id is dropped. */
export class DeckLoader {
	deck = $state<Deck | null>(null);
	missing = $state(false);
	private wanted = '';

	async load(id: string): Promise<void> {
		this.wanted = id;
		this.deck = null;
		this.missing = false;
		try {
			const found = await backend.deck(id);
			if (this.wanted === id) this.deck = found;
		} catch {
			if (this.wanted === id) this.missing = true;
		}
	}
}
