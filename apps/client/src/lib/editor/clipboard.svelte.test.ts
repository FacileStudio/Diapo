import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { plainText } from '../components/editor/text';
import { EditorActions } from './actions';
import { CLIP_PREFIX, decodeShapes, encodeShapes } from './clipboard';
import { localDeckEditor, type DeckEditorState } from './deck.svelte';
import { problems } from './testing';

function write(editor: DeckEditorState, id: string, words: string) {
	const paragraph = editor.text(id)?.get(0) as Y.XmlElement;
	editor.doc.transact(() => {
		const run = new Y.XmlText();
		paragraph.insert(0, [run]);
		run.insert(0, words, { bold: true });
	});
}

describe('clipboard', () => {
	it('copies shapes with their rich text into another deck, on top, and pastes again offset', () => {
		const source = localDeckEditor();
		const text = source.insert('text', { x: 100, y: 100 });
		write(source, text, 'Bonjour');
		const box = source.insert('rect', { x: 300, y: 300 });
		source.update(box, { style: { fill: '#f00', align: 'justify', valign: 'bottom' }, step: 2 });
		source.select([text, box]);
		const payload = new EditorActions(source).copy();
		expect(payload.startsWith(CLIP_PREFIX)).toBe(true);

		const target = localDeckEditor();
		target.insert('ellipse');
		const actions = new EditorActions(target);
		const [a, b] = actions.pasteText(payload);
		const shapes = target.slides[0].shapes;
		expect(shapes.map((shape) => shape.id)).toEqual([shapes[0].id, a, b]);
		expect(shapes[1]).toMatchObject({ type: 'text', x: 100, y: 100, parent: target.current });
		expect(shapes[2]).toMatchObject({ type: 'rect', step: 2, style: { fill: '#f00', align: 'justify', valign: 'bottom' } });
		expect(plainText(target.text(a))).toBe('Bonjour');
		expect(target.selection).toEqual([a, b]);

		const [again] = actions.pasteText(payload);
		expect(target.slides[0].shapes.find((shape) => shape.id === again)).toMatchObject({ x: 140, y: 140 });
		const [centred] = actions.pasteText(payload, { x: 1000, y: 1000 });
		expect(target.slides[0].shapes.find((shape) => shape.id === centred)).toMatchObject({ x: 520, y: 740 });
		expect(problems(target.doc)).toEqual([]);
	});

	it('cuts, pastes plain text as a text box and refuses what a full server would refuse', () => {
		const editor = localDeckEditor();
		const box = editor.insert('rect');
		editor.select([box]);
		const actions = new EditorActions(editor);
		const payload = actions.cut();
		expect(editor.slides[0].shapes).toEqual([]);
		expect(encodeShapes(editor.doc, [box])).toBe(`${CLIP_PREFIX}{"v":1,"shapes":[]}`);
		expect(actions.pasteText(payload)).toHaveLength(1);
		const [plain] = actions.pasteText('Two\nlines');
		expect(plainText(editor.text(plain))).toBe('Two\nlines');

		const hostile = CLIP_PREFIX + JSON.stringify({
			v: 1,
			shapes: [
				{ id: 'a', type: 'frame', fields: {} },
				{ id: 'b', type: 'nope', fields: {} },
				{
					id: 'c',
					type: 'text',
					fields: { x: 'big', bogus: 1, style: { fill: 3, colour: 'red' } },
					text: [{ type: 'script', content: [] }, { type: 'paragraph', attrs: { onclick: 'x' }, content: [{ text: 'ok', marks: { evil: {} } }] }]
				}
			]
		});
		expect(decodeShapes(hostile)?.shapes.map((shape) => shape.id)).toEqual(['c']);
		const [kept] = actions.pasteText(hostile);
		expect(plainText(editor.text(kept))).toBe('ok');
		expect(editor.doc.shapes.get(kept)?.has('bogus')).toBe(false);
		expect(decodeShapes(`${CLIP_PREFIX}{oops`)).toBeNull();
		expect(problems(editor.doc)).toEqual([]);
	});
});
