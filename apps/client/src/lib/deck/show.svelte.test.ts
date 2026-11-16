import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Show, Stopwatch, clock } from './show.svelte';
import type { StageInfo } from './stage';

const INFO: StageInfo = {
	duration: 12,
	steps: 4,
	scenes: [
		{ start: 0, duration: 8, steps: 2, first: 0 },
		{ start: 8, duration: 4, steps: 1, first: 3 }
	]
};

describe('Show', () => {
	it('steps through reveals before changing scene, and stops at both ends', () => {
		const show = new Show();
		show.playhead.ready(INFO);
		expect(show.playhead.want).toBe(0);
		for (const key of ['ArrowRight', ' ', 'PageDown']) expect(show.key(key)).toBe(true);
		expect(show.playhead.want).toBe(3);
		show.key('End');
		show.key('ArrowRight');
		expect(show.playhead.want).toBe(4);
		show.key('Home');
		show.key('ArrowLeft');
		expect(show.playhead.want).toBe(0);
		expect(show.key('f')).toBe(false);
	});

	it('keeps a step another window sent before the document was ready, then clamps it', () => {
		const show = new Show();
		show.playhead.go(3);
		show.playhead.ready(INFO);
		expect(show.playhead.want).toBe(3);
		show.playhead.go(40);
		expect(show.playhead.want).toBe(4);
	});

	it('names the shown scene and step for people', () => {
		const show = new Show();
		show.playhead.ready(INFO);
		show.playhead.drawn({ time: 9, step: 4, scene: 1 });
		expect(show.playhead.position).toEqual({ scene: 2, scenes: 2, step: 1, steps: 1 });
	});
});

describe('Stopwatch', () => {
	beforeEach(() => vi.useFakeTimers());
	afterEach(() => vi.useRealTimers());

	it('counts while running, keeps its time across a pause, and resets', () => {
		const watch = new Stopwatch();
		watch.toggle();
		vi.advanceTimersByTime(61_000);
		watch.toggle();
		vi.advanceTimersByTime(5_000);
		expect(clock(watch.elapsed)).toBe('01:01');
		watch.toggle();
		vi.advanceTimersByTime(3_600_000);
		expect(clock(watch.elapsed)).toBe('1:01:01');
		watch.reset();
		expect([watch.elapsed, watch.running]).toEqual([0, false]);
	});
});
