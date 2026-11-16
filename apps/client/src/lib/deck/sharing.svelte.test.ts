import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import { Sharing, embedSnippet, shareToken } from './sharing.svelte';

afterEach(() => vi.restoreAllMocks());

const URL_ONCE = 'https://diapo.test/p/tok_EN-1';

describe('Sharing', () => {
	it('reads the state, and shows the URL only from the call that created the link', async () => {
		vi.spyOn(backend, 'share').mockResolvedValue({ shared: false, created_at: null });
		const create = vi
			.spyOn(backend, 'createShare')
			.mockResolvedValue({ shared: true, created_at: 'now', url: URL_ONCE });
		const sharing = new Sharing('d1');
		expect(sharing.status).toBe('loading');
		await sharing.load();
		expect([sharing.status, sharing.url]).toEqual(['off', null]);
		await sharing.create();
		expect([sharing.status, sharing.url, create.mock.calls[0]]).toEqual(['on', URL_ONCE, ['d1', false]]);
		sharing.forget();
		expect([sharing.status, sharing.url]).toEqual(['on', null]);
	});

	it('knows a deck already shared has no URL to show', async () => {
		vi.spyOn(backend, 'share').mockResolvedValue({ shared: true, created_at: 'then' });
		vi.spyOn(backend, 'createShare').mockResolvedValue({ shared: true, created_at: 'then' });
		const sharing = new Sharing('d1');
		await sharing.load();
		await sharing.create();
		expect([sharing.status, sharing.url, sharing.createdAt]).toEqual(['on', null, 'then']);
	});

	it('rotates to a new URL and revokes back to off', async () => {
		const create = vi
			.spyOn(backend, 'createShare')
			.mockResolvedValue({ shared: true, created_at: 'now', url: URL_ONCE });
		vi.spyOn(backend, 'revokeShare').mockResolvedValue(undefined);
		const sharing = new Sharing('d1');
		await sharing.create(true);
		expect([create.mock.calls[0], sharing.url]).toEqual([['d1', true], URL_ONCE]);
		await sharing.revoke();
		expect([sharing.status, sharing.url, sharing.createdAt]).toEqual(['off', null, null]);
	});

	it('keeps its state and frees the buttons when a call fails, and marks a failed read', async () => {
		vi.spyOn(backend, 'share').mockRejectedValue(new ApiError(0, 'network', 'offline'));
		vi.spyOn(backend, 'revokeShare').mockRejectedValue(new ApiError(404, 'not_found', 'not shared'));
		const sharing = new Sharing('d1');
		await sharing.load();
		expect(sharing.status).toBe('failed');
		sharing.status = 'on';
		await expect(sharing.revoke()).rejects.toMatchObject({ status: 404 });
		expect([sharing.status, sharing.busy]).toEqual(['on', false]);
	});
});

describe('embedSnippet', () => {
	it('frames /embed/<token> of the same origin at the stage ratio', () => {
		expect(shareToken(URL_ONCE)).toBe('tok_EN-1');
		expect(embedSnippet(URL_ONCE, 'Q3 "review" <b>', 1920, 1080)).toBe(
			'<iframe src="https://diapo.test/embed/tok_EN-1" width="960" height="540" title="Q3 review b" frameborder="0" allowfullscreen></iframe>'
		);
	});

	it('answers nothing for a URL that is not a share link', () => {
		expect([shareToken('https://diapo.test/decks/d1'), shareToken('nope')]).toEqual([null, null]);
		expect(embedSnippet('https://diapo.test/decks/d1', 'x', 16, 9)).toBe('');
	});
});
