import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import { Viewing } from './viewing.svelte';

afterEach(() => vi.restoreAllMocks());

const DECK = {
	title: 'Q3',
	scenes: 2,
	steps: 3,
	width: 1920,
	height: 1080,
	rev: 4,
	frame: '/api/public/decks/tok/frame'
};

describe('Viewing', () => {
	it('frames the public document at its revision and steps with the keyboard', async () => {
		const read = vi.spyOn(backend, 'publicDeck').mockResolvedValue(DECK);
		const viewing = new Viewing();
		await viewing.load('tok');
		expect([read.mock.calls[0][0], viewing.src]).toEqual(['tok', '/api/public/decks/tok/frame?v=4']);
		expect([viewing.key('ArrowRight'), viewing.playhead.want, viewing.key('x')]).toEqual([true, 1, false]);
		viewing.key('End');
		expect(viewing.playhead.want).toBe(3);
	});

	it('calls a link dead only on a 404, and lets any other failure be tried again', async () => {
		const read = vi.spyOn(backend, 'publicDeck').mockRejectedValueOnce(new ApiError(404, 'not_found', 'gone'));
		const dead = new Viewing();
		await dead.load('tok');
		expect([dead.gone, dead.failed]).toEqual([true, false]);
		read.mockRejectedValueOnce(new ApiError(429, 'rate_limited', 'slow down')).mockResolvedValueOnce(DECK);
		const busy = new Viewing();
		await busy.load('tok');
		expect([busy.gone, busy.failed]).toEqual([false, true]);
		await busy.load('tok');
		expect([busy.failed, busy.deck?.title]).toEqual([false, 'Q3']);
	});

	it('reads the public route without the session cookie', async () => {
		const fetched = vi.fn(async (_url: string, _init: RequestInit) => new Response(JSON.stringify(DECK)));
		vi.stubGlobal('fetch', fetched);
		await backend.publicDeck('tok');
		expect([fetched.mock.calls[0][0], fetched.mock.calls[0][1].credentials]).toEqual(['/api/public/decks/tok', 'omit']);
		vi.unstubAllGlobals();
	});
});
