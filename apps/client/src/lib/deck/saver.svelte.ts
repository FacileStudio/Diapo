import { ApiError } from '$lib/backend';
import type { SceneError } from '$lib/types';

/** Where the buffer stands against the stored deck. */
export type SaveStatus = 'clean' | 'dirty' | 'saving' | 'invalid' | 'conflict' | 'failed';

/** The two calls the saver makes, and what it tells the editor. */
export interface SaverIo {
	put(source: string, rev: number): Promise<{ rev: number }>;
	get(): Promise<{ source: string; rev: number }>;
	/** The stored source replaced the buffer: the editor must show `source`. */
	onreplace(source: string): void;
}

/**
 * Saves a deck source with its revision. An edit is saved after `delay` ms; a stale revision or
 * an outside write over unsaved text stops in `conflict` until the person picks a side; a source
 * the parser refuses stays in `invalid` with its errors until the next edit.
 */
export class Saver {
	status = $state<SaveStatus>('clean');
	rev = $state(0);
	errors = $state<SceneError[]>([]);
	buffer = '';
	private timer: ReturnType<typeof setTimeout> | null = null;
	private seen = 0;
	private echo = 0;

	constructor(
		private readonly io: SaverIo,
		private readonly delay = 700
	) {}

	open(deck: { source: string; rev: number }): void {
		this.cancel();
		this.buffer = deck.source;
		this.rev = deck.rev;
		this.errors = [];
		this.status = 'clean';
	}

	edit(source: string): void {
		this.buffer = source;
		if (this.status === 'conflict') return;
		if (this.status !== 'saving') this.status = 'dirty';
		this.cancel();
		this.timer = setTimeout(() => void this.flush(), this.delay);
	}

	async flush(): Promise<void> {
		this.cancel();
		if (this.status === 'saving' || this.status === 'conflict' || this.status === 'clean') return;
		const sent = this.buffer;
		this.status = 'saving';
		try {
			this.rev = (await this.io.put(sent, this.rev)).rev;
			this.echo = this.seen >= this.rev ? 0 : this.rev;
			this.errors = [];
			this.settle(sent);
		} catch (error) {
			this.refused(error);
		}
	}

	/** An outside write reached revision `rev`, as the `changed` event reports it. */
	async remote(rev: number): Promise<void> {
		this.seen = Math.max(this.seen, rev);
		if (this.status === 'saving' || this.seen <= this.rev) return;
		if (this.status === 'clean') await this.reload();
		else this.stop();
	}

	/**
	 * Whether a `changed` event at `rev` reports a change that is not a new source: a rename or
	 * an asset. The event that echoes this editor's own save is not one.
	 */
	touched(rev: number): boolean {
		if (rev !== this.rev || this.status === 'saving') return false;
		const own = this.echo === rev;
		this.echo = 0;
		return !own;
	}

	/** Drops the buffer for the stored source. */
	async reload(): Promise<void> {
		const theirs = await this.io.get();
		this.open(theirs);
		this.io.onreplace(theirs.source);
	}

	/** Keeps the buffer and writes it over the stored source. */
	async overwrite(): Promise<void> {
		this.rev = (await this.io.get()).rev;
		this.status = 'dirty';
		await this.flush();
	}

	dispose(): void {
		this.cancel();
	}

	private settle(sent: string): void {
		const clean = this.buffer === sent;
		const moved = this.seen > this.rev;
		if (moved && !clean) return this.stop();
		this.status = clean ? 'clean' : 'dirty';
		if (!clean) this.edit(this.buffer);
		else if (moved) void this.reload();
	}

	private refused(error: unknown): void {
		const status = error instanceof ApiError ? error.status : 0;
		if (status === 409) return this.stop();
		this.errors = status === 422 && error instanceof ApiError ? error.errors : [];
		this.status = status === 422 ? 'invalid' : 'failed';
	}

	private stop(): void {
		this.cancel();
		this.status = 'conflict';
	}

	private cancel(): void {
		if (this.timer) clearTimeout(this.timer);
		this.timer = null;
	}
}
