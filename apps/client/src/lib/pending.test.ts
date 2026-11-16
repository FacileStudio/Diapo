import { describe, expect, it } from 'vitest';
import { ApiError } from './backend';
import { STATES, needsBrowserLogin } from './pending';

describe('pending', () => {
	it('asks for a browser sign-in only on the 403 of a session that is not one', () => {
		expect(needsBrowserLogin(new ApiError(403, 'permission_denied', 'own session only'))).toBe(true);
		expect(needsBrowserLogin(new ApiError(404, 'not_found', 'gone'))).toBe(false);
		expect(needsBrowserLogin(new Error('offline'))).toBe(false);
	});

	it('describes the three states the API names', () => {
		expect(Object.keys(STATES)).toEqual(['ready', 'changed', 'missing']);
	});
});
