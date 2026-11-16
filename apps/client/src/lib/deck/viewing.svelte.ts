import { ApiError, backend } from '$lib/backend';
import type { PublicDeck } from '$lib/types';
import { Playhead } from './playhead.svelte';
import { keyStep } from './stage';

/**
 * A shared deck being watched through its link. It calls the public API only, without a session.
 * Only a `404` means the link is dead; any other failure can be tried again.
 */
export class Viewing {
	deck = $state<PublicDeck | null>(null);
	gone = $state(false);
	failed = $state(false);
	readonly playhead = new Playhead(false);

	async load(token: string): Promise<void> {
		this.failed = false;
		try {
			this.deck = await backend.publicDeck(token);
		} catch (error) {
			this.gone = error instanceof ApiError && error.status === 404;
			this.failed = !this.gone;
		}
	}

	/** The URL of the sandboxed document; it changes with the revision so a new one loads. */
	get src(): string {
		return this.deck ? `${this.deck.frame}?v=${this.deck.rev}` : '';
	}

	/** Moves as `key` asks; false when the key moves nothing. */
	key(key: string): boolean {
		const next = keyStep(key, this.playhead.want, this.playhead.info?.steps ?? this.deck?.steps ?? 0);
		if (next === null) return false;
		this.playhead.go(next);
		return true;
	}
}
