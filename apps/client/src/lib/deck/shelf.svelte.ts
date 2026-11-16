import { backend } from '$lib/backend';
import type { DeckSummary } from '$lib/types';

/** The decks of the signed-in person, newest first, or their trash, and the writes the list pages make. */
export class Shelf {
	decks = $state<DeckSummary[]>([]);
	ready = $state(false);
	failed = $state(false);

	constructor(private readonly trashed = false) {}

	async load(): Promise<void> {
		try {
			this.decks = await (this.trashed ? backend.trashed() : backend.decks());
			this.failed = false;
		} catch {
			this.failed = true;
		}
		this.ready = true;
	}

	create(title: string): Promise<DeckSummary> {
		return backend.create(title);
	}

	async rename(id: string, title: string): Promise<void> {
		const renamed = await backend.rename(id, title);
		this.decks = this.decks.map((deck) => (deck.id === id ? { ...deck, title: renamed.title } : deck));
	}

	async trash(id: string): Promise<void> {
		await backend.trash(id, true);
		this.decks = this.decks.filter((deck) => deck.id !== id);
	}

	async restore(id: string): Promise<void> {
		await backend.trash(id, false);
		await this.load();
	}

	/** Deletes a trashed deck for good. */
	async destroy(id: string): Promise<void> {
		await backend.destroy(id);
		this.decks = this.decks.filter((deck) => deck.id !== id);
	}
}
