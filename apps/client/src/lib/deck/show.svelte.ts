import { Playhead } from './playhead.svelte';
import { joinRemote } from './remote';
import { keyStep } from './stage';

/** A deck being presented in this window, in step with every other window presenting it. */
export class Show {
	readonly playhead = new Playhead(false);
	private remote: ReturnType<typeof joinRemote> | null = null;

	/** Joins the other windows showing deck `id`; answers the function that leaves. */
	join(id: string): () => void {
		const remote = joinRemote(id, { current: () => this.playhead.want, onstep: (k) => this.playhead.go(k) });
		this.remote = remote;
		return () => remote.close();
	}

	go(k: number): void {
		this.playhead.go(k);
		this.remote?.send(this.playhead.want);
	}

	move(delta: number): void {
		this.go(this.playhead.want + delta);
	}

	/** Moves as `key` asks; false when the key moves nothing. */
	key(key: string): boolean {
		const next = keyStep(key, this.playhead.want, this.playhead.info?.steps ?? 0);
		if (next === null) return false;
		this.go(next);
		return true;
	}
}

/** A stopwatch for the presenter view. */
export class Stopwatch {
	elapsed = $state(0);
	running = $state(false);
	private since = 0;
	private before = 0;
	private timer: ReturnType<typeof setInterval> | null = null;

	toggle(): void {
		if (this.running) return this.pause();
		this.since = Date.now();
		this.running = true;
		this.timer = setInterval(() => (this.elapsed = this.before + Date.now() - this.since), 250);
	}

	pause(): void {
		if (this.timer) clearInterval(this.timer);
		this.timer = null;
		if (this.running) this.before += Date.now() - this.since;
		this.elapsed = this.before;
		this.running = false;
	}

	reset(): void {
		this.pause();
		this.before = 0;
		this.elapsed = 0;
	}
}

/** `ms` as a clock: `07:42`, or `1:07:42` past an hour. */
export function clock(ms: number): string {
	const total = Math.floor(ms / 1000);
	const pad = (value: number) => String(value).padStart(2, '0');
	const hours = Math.floor(total / 3600);
	const rest = `${pad(Math.floor((total % 3600) / 60))}:${pad(total % 60)}`;
	return hours > 0 ? `${hours}:${rest}` : rest;
}
