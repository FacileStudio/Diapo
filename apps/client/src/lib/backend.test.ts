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
