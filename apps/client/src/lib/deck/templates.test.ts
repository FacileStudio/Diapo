import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import { bindLocale } from '$lib/i18n';
import { isPlanche, readPlan } from './projection';
import { Shelf } from './shelf.svelte';
import { starterSource } from './templates';

afterEach(() => {
	vi.restoreAllMocks();
	bindLocale(() => 'fr');
});

describe('starterSource', () => {
	it('is a planche cover and a two-step slide with a note, in the person’s language', () => {
		const source = starterSource('Bilan T3');
		expect(isPlanche(source)).toBe(true);
		const plan = readPlan(source);
		expect([plan.width, plan.height, plan.notes.length]).toEqual([1920, 1080, 2]);
		expect(plan.notes[0]).toBe('');
		expect(plan.notes[1]).toContain('vue de l’orateur');
		expect(source).toContain('# Bilan T3\n');
		expect(source.match(/step="\d"/g)).toEqual(['step="1"', 'step="2"']);
		bindLocale(() => 'de');
		expect(starterSource('Q3')).toContain('## Erstes Thema\n');
	});

	it('keeps a title from breaking an attribute or the markup', () => {
		const source = starterSource('Plan "A": <b>go</b>\n`x`');
		expect(source).toContain('name="Plan  A :  b go /b   x"');
		expect(readPlan(source).notes).toHaveLength(2);
	});
});

describe('readPlan', () => {
	it('reads a legacy source by its scenes', () => {
		const plan = readPlan('---\nsize: 1280x720\n---\n\n# A\n\n---\n\n## B\n\n<!-- Say B. -->\n');
		expect(plan).toEqual({ width: 1280, height: 720, notes: ['', 'Say B.'] });
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
