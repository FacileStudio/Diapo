import { describe, expect, it } from 'vitest';
import { outline, sceneAt } from './outline';

const DECK = [
	'---',
	'title: Q3 review',
	'size: 1280x720',
	'layout: cover',
	'---',
	'',
	'# Q3 review',
	'',
	'<!-- Thank the team first. -->',
	'',
	'---',
	'layout: split',
	'---',
	'',
	'## Revenue',
	'',
	'- Up 12% {step=1}',
	'- Churn down {step=2}',
	'',
	'<!--',
	'Two lines',
	'of notes.',
	'-->',
	'---',
	'',
	'```md',
	'---',
	'```',
	'',
	'Not a note: <!-- inline -->',
	'---'
].join('\n');

describe('outline', () => {
	it('splits scenes, reads the stage size, the layouts and the step counts', () => {
		const found = outline(DECK);
		expect([found.width, found.height]).toEqual([1280, 720]);
		expect(found.scenes.map((scene) => [scene.from, scene.to, scene.layout, scene.steps])).toEqual([
			[1, 10, 'cover', 0],
			[11, 23, 'split', 2],
			[24, 30, 'default', 0]
		]);
		expect(found.scenes.map((scene) => scene.title)).toEqual(['Q3 review', 'Revenue', '```md']);
	});

	it('reads the trailing comment of a scene as its presenter note, and no other comment', () => {
		const notes = outline(DECK).scenes.map((scene) => scene.note);
		expect(notes).toEqual(['Thank the team first.', 'Two lines\nof notes.', '']);
		expect(outline('# A\n\n<!-- one -->\n\ntext\n').scenes[0].note).toBe('');
		expect(outline('# A\n\n<!-- one -->\n<!-- two -->').scenes[0].note).toBe('two');
	});

	it('takes the defaults without a headmatter, and a declared step count over the markers', () => {
		const found = outline(
			'# One\r\n\r\n---\r\nsteps: 5\r\n---\r\n\r\nA {step=1}\r\n\r\n---\r\n\r\nkey: not frontmatter'
		);
		expect([found.width, found.height]).toEqual([1920, 1080]);
		expect(found.scenes.map((scene) => scene.steps)).toEqual([0, 5, 0]);
		expect(found.scenes[2].title).toBe('key: not frontmatter');
	});

	it('reads `---` followed by `---` as an empty frontmatter', () => {
		expect(outline('# A\n---\n---\n# B').scenes.map((scene) => scene.title)).toEqual(['A', 'B']);
	});

	it('finds the scene of a line', () => {
		const { scenes } = outline(DECK);
		expect([1, 10, 11, 18, 24, 40].map((line) => sceneAt(scenes, line))).toEqual([0, 0, 1, 1, 2, 2]);
	});
});
