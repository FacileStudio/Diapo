import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend, request } from './backend';

function answer(status: number, body: unknown): Response {
	return new Response(body === undefined ? null : JSON.stringify(body), { status });
}

afterEach(() => vi.unstubAllGlobals());

describe('request', () => {
	it('sends the CSRF header on a mutation and not on a read', async () => {
		const fetched = vi.fn(async (_url: string, _init: RequestInit) => answer(200, {}));
		vi.stubGlobal('fetch', fetched);
		await backend.logout();
		await backend.session();
		const headers = fetched.mock.calls.map(([, init]) => new Headers(init.headers).get('X-Facile-CSRF'));
		expect(headers).toEqual(['1', null]);
		expect(fetched.mock.calls[0][0]).toBe('/api/auth/logout');
		expect(fetched.mock.calls[0][1].credentials).toBe('include');
	});

	it('throws the suite envelope as a typed error', async () => {
		vi.stubGlobal('fetch', async () => answer(404, { error: { code: 'not_found', message: 'route not found' } }));
		const error = await request('/nope').catch((caught: unknown) => caught);
		expect(error).toBeInstanceOf(ApiError);
		expect(error).toMatchObject({ status: 404, code: 'not_found', message: 'route not found' });
	});

	it('names a network failure', async () => {
		vi.stubGlobal('fetch', async () => Promise.reject(new TypeError('offline')));
		await expect(request('/auth/session')).rejects.toMatchObject({ status: 0, code: 'network' });
	});
});

describe('decks', () => {
	it('saves a source with the revision it read', async () => {
		const fetched = vi.fn(async (_url: string, _init: RequestInit) => answer(200, { rev: 8 }));
		vi.stubGlobal('fetch', fetched);
		expect(await backend.save('d 1', '# A', 7)).toEqual({ rev: 8 });
		const [url, init] = fetched.mock.calls[0];
		expect([url, init.method, init.body]).toEqual(['/api/decks/d%201/source', 'PUT', '{"source":"# A","rev":7}']);
		expect(new Headers(init.headers).get('X-Facile-CSRF')).toBe('1');
	});

	it('names a stale revision a conflict', async () => {
		vi.stubGlobal('fetch', async () =>
			answer(409, { error: { code: 'already_exists', message: 'the deck is at rev 5, not 3' }, rev: 5 })
		);
		await expect(backend.save('d1', '', 1)).rejects.toMatchObject({ status: 409, code: 'already_exists', errors: [] });
	});

	it('reads the scene errors of a refused source, wherever the envelope carries them', async () => {
		const errors = [{ line: 4, message: 'step 2 is past `steps: 1`' }];
		const envelope = { code: 'invalid_argument', message: 'invalid source' };
		vi.stubGlobal('fetch', async () => answer(422, { error: { ...envelope, errors } }));
		await expect(backend.save('d1', '', 1)).rejects.toMatchObject({ status: 422, errors });
		vi.stubGlobal('fetch', async () => answer(422, { error: envelope, errors: [...errors, { line: 'x' }] }));
		await expect(backend.save('d1', '', 1)).rejects.toMatchObject({ errors });
	});

	it('trashes with a patch and uploads an asset as multipart', async () => {
		const fetched = vi.fn(async (_url: string, _init: RequestInit) => answer(200, {}));
		vi.stubGlobal('fetch', fetched);
		await backend.trash('d1', true);
		await backend.upload('d1', new File(['x'], 'chart.png', { type: 'image/png' }));
		await backend.removeAsset('d1', 'assets/chart.png');
		const [patch, post, remove] = fetched.mock.calls;
		expect([patch[1].method, patch[1].body]).toEqual(['PATCH', '{"trashed":true}']);
		expect((post[1].body as FormData).get('file')).toBeInstanceOf(File);
		expect(new Headers(post[1].headers).has('Content-Type')).toBe(false);
		expect([remove[0], remove[1].method]).toEqual(['/api/decks/d1/assets/assets/chart.png', 'DELETE']);
	});

	it('builds the frame and events URLs', () => {
		expect(backend.frameUrl('d1')).toBe('/api/decks/d1/frame');
		expect(backend.frameUrl('d1', { scene: 0, stamp: '3.1' })).toBe('/api/decks/d1/frame?scene=0&v=3.1');
		expect(backend.eventsUrl('d1')).toBe('/api/decks/d1/events');
	});
});
