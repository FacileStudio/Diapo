import { describe, expect, it } from 'vitest';
import { joinRemote } from './remote';

const settle = () => new Promise((resolve) => setTimeout(resolve, 20));

describe('joinRemote', () => {
	it('hands a joining window the current step, then mirrors every move', async () => {
		const steps = { present: 4, presenter: 0 };
		const present = joinRemote('d1', { current: () => steps.present, onstep: (k) => (steps.present = k) });
		const presenter = joinRemote('d1', { current: () => steps.presenter, onstep: (k) => (steps.presenter = k) });
		await settle();
		expect(steps).toEqual({ present: 4, presenter: 4 });
		presenter.send(6);
		await settle();
		expect(steps.present).toBe(6);
		present.close();
		presenter.close();
	});

	it('keeps decks apart and ignores malformed messages', async () => {
		let step = 1;
		const mine = joinRemote('d1', { current: () => step, onstep: (k) => (step = k) });
		const other = joinRemote('d2', { current: () => 9, onstep: () => undefined });
		const raw = new BroadcastChannel('diapo.present.d1');
		raw.postMessage({ type: 'step', k: 'x' });
		other.send(7);
		await settle();
		expect(step).toBe(1);
		for (const channel of [mine, other, raw]) channel.close();
	});
});
