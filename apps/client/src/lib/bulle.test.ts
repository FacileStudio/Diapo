import { afterEach, describe, expect, it, vi } from 'vitest';
import { backend } from './backend';
import { DEFAULT_BULLE, assistant, bulleMessage, embedUrl, rememberSidebar, sidebarOpen, trustedOrigin } from './bulle';

afterEach(() => vi.restoreAllMocks());

describe('bulle', () => {
	it('builds the chat embed URL of a deck', () => {
		const page = { app: 'diapo' as const, id: 'd1', title: 'Plan & co', url: 'https://diapo.facile.studio/decks/d1' };
		const url = new URL(embedUrl(page, DEFAULT_BULLE));
		expect(url.origin + url.pathname).toBe('https://bulle.facile.studio/embed/chat');
		expect(Object.fromEntries(url.searchParams)).toEqual(page);
	});

	it('trusts the configured Bulle, and localhost only while developing', () => {
		expect(trustedOrigin('https://bulle.facile.studio', DEFAULT_BULLE)).toBe(true);
		expect(trustedOrigin('https://bulle.facile.studio', 'https://bulle.test')).toBe(false);
		expect(trustedOrigin('https://evil.example', DEFAULT_BULLE)).toBe(false);
		expect(trustedOrigin('http://localhost:5173', DEFAULT_BULLE)).toBe(false);
		expect(trustedOrigin('http://localhost:5173', DEFAULT_BULLE, true)).toBe(true);
	});

	it('reads only the two messages Bulle sends', () => {
		expect(bulleMessage({ type: 'bulle:close' })).toEqual({ type: 'bulle:close' });
		expect(bulleMessage({ type: 'bulle:ready', assistant: { name: 'Nova' } })?.type).toBe('bulle:ready');
		expect(bulleMessage({ type: 'bulle:ready' })).toBeNull();
		expect(bulleMessage('bulle:close')).toBeNull();
	});

	it('remembers the sidebar per person', () => {
		rememberSidebar('a@x', true);
		expect([sidebarOpen('a@x'), sidebarOpen('b@x')]).toEqual([true, false]);
	});

	it('asks the assistant once and falls back to Bulle, asking again after a failure', async () => {
		const ask = vi.spyOn(backend, 'assistant').mockRejectedValueOnce(new Error('down'));
		expect((await assistant()).name).toBe('Bulle');
		ask.mockResolvedValue({ name: 'Nova', accent: null });
		expect((await assistant()).name).toBe('Nova');
		expect((await assistant()).name).toBe('Nova');
		expect(ask).toHaveBeenCalledTimes(2);
	});
});
