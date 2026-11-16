import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import { bindLocale } from '$lib/i18n';
import { outline } from './outline';
import { Shelf } from './shelf.svelte';
import { starterSource } from './templates';

afterEach(() => {
	vi.restoreAllMocks();
	bindLocale(() => 'fr');
});

describe('starterSource', () => {
	it('is a cover and a two-step scene with a note, in the person’s language', () => {
		const found = outline(starterSource('Bilan T3'));
		expect(found.scenes.map((scene) => [scene.layout, scene.title, scene.steps])).toEqual([
			['cover', 'Bilan T3', 0],
			['default', 'Premier sujet', 2]
		]);
		expect(found.scenes[1].note).toContain('vue de l’orateur');
		bindLocale(() => 'de');
		expect(outline(starterSource('Q3')).scenes[1].title).toBe('Erstes Thema');
	});

	it('keeps a title from breaking the frontmatter or the markup', () => {
		const source = starterSource('Plan "A": <b>go</b>\n---');
		expect(source.split('\n')[1]).toBe('title: "Plan  A :  b go /b  ---"');
		expect(outline(source).scenes).toHaveLength(2);
	});
});

describe('Shelf.create', () => {
	it('sends the starter, and falls back to the API’s own when the source is refused', async () => {
		const create = vi.spyOn(backend, 'create');
		const deck = { id: 'd1', title: 'T', rev: 1, updated_at: '', source: '' };
		create.mockRejectedValueOnce(new ApiError(422, 'invalid_argument', 'bad')).mockResolvedValueOnce(deck);
		expect(await new Shelf().create('T')).toBe(deck);
		expect(create.mock.calls.map((call) => call.length)).toEqual([2, 1]);
		expect(create.mock.calls[0][1]).toContain('# T');
		create.mockRejectedValueOnce(new ApiError(403, 'permission_denied', 'no'));
		await expect(new Shelf().create('T')).rejects.toMatchObject({ status: 403 });
	});
});
