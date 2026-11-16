import { describe, expect, it } from 'vitest';
import { outline } from '$lib/deck/outline';
import { coverLines, sceneLines, typedScene } from './demo-source';

describe('the landing demo source', () => {
	it('is a deck the real format reads as a cover and a two-point scene', () => {
		const typed = sceneLines('Revenue', 'Up 12%', 'Churn down');
		const found = outline([...coverLines('Q3 review'), ...typed].join('\n'));
		expect(found.scenes.map((scene) => [scene.layout, scene.title])).toEqual([
			['cover', 'Q3 review'],
			['default', 'Revenue']
		]);
		expect(typedScene(typed)).toEqual({ open: true, heading: 'Revenue', bullets: ['Up 12%', 'Churn down'] });
	});

	it('draws nothing of the second scene until its separator is whole', () => {
		expect(typedScene(['--'])).toEqual({ open: false, heading: '', bullets: [] });
		expect(typedScene(['---', '', '## Rev'])).toEqual({ open: true, heading: 'Rev', bullets: [] });
	});
});
