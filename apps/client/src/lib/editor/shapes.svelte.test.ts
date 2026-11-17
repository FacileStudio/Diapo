import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { localDeckEditor } from './deck.svelte';
import { duplicateShapes } from './shapes';
import { problems } from './testing';

describe('shapes', () => {
	it('inserts centred defaults, rich text and contained media on the current slide', () => {
		const editor = localDeckEditor();
		const text = editor.insert('text');
		const slide = editor.current;
		expect([editor.editing, editor.selection]).toEqual([text, [text]]);
		const image = editor.insert('image', { x: 10 }, 'assets/a.png');
		const video = editor.insert('video', undefined, 'assets/b.mp4');
		const [a, b, c] = editor.slides[0].shapes;
		expect([a.id, b.id, c.id]).toEqual([text, image, video]);
		expect(a).toMatchObject({ parent: slide, x: 480, y: 460, w: 960, h: 160 });
		expect(b).toMatchObject({ x: 10, asset: 'assets/a.png', fit: 'contain' });
		expect(editor.text(text)).toBeInstanceOf(Y.XmlFragment);
		expect(editor.text(image)).toBeUndefined();
		expect([editor.editing, editor.selection]).toEqual([null, [video]]);
		expect(problems(editor.doc)).toEqual([]);
	});

	it('updates allowed keys only and removes shapes from the selection', () => {
		const editor = localDeckEditor();
		const box = editor.insert('rect');
		editor.update(box, { x: 5, step: 2, opacity: 3, name: 'no', fit: 'cover', locked: true });
		expect(editor.slides[0].shapes[0]).toMatchObject({ x: 5, step: 2, opacity: 1, fit: 'cover', locked: true });
		expect(editor.doc.shapes.get(box)?.has('name')).toBe(false);
		editor.update(box, { locked: undefined });
		expect(editor.doc.shapes.get(box)?.has('locked')).toBe(false);
		editor.remove([box]);
		expect(editor.slides[0].shapes).toEqual([]);
		expect(editor.selection).toEqual([]);
		expect(problems(editor.doc)).toEqual([]);
	});

	it('reorders in paint order and duplicates on top', () => {
		const editor = localDeckEditor();
		const [a, b, c] = [editor.insert('rect'), editor.insert('ellipse'), editor.insert('diamond')];
		const ids = () => editor.slides[0].shapes.map((shape) => shape.id);
		editor.reorder(a, 'front');
		expect(ids()).toEqual([b, c, a]);
		editor.reorder(a, 'back');
		expect(ids()).toEqual([a, b, c]);
		editor.reorder(a, 'forward');
		expect(ids()).toEqual([b, a, c]);
		editor.reorder(c, 'backward');
		expect(ids()).toEqual([b, c, a]);
		const [copy] = duplicateShapes(editor.doc, [b]);
		expect(ids().at(-1)).toBe(copy);
		expect(editor.text(copy)).not.toBe(editor.text(b));
		expect(problems(editor.doc)).toEqual([]);
	});

	it('undoes and redoes only local edits', () => {
		const editor = localDeckEditor();
		const box = editor.insert('rect');
		editor.update(box, { x: 1 });
		expect(editor.canUndo).toBe(true);
		editor.undo();
		expect(editor.slides).toEqual([]);
		expect(editor.canRedo).toBe(true);
		editor.redo();
		expect(editor.slides[0].shapes[0]).toMatchObject({ id: box, x: 1 });
		editor.doc.ydoc.transact(() => editor.doc.shapes.get(box)?.set('x', 9), 'peer');
		editor.undo();
		expect(editor.slides).toEqual([]);
		editor.redo();
		expect(editor.slides[0].shapes[0]).toMatchObject({ x: 9 });
	});

	it('reads peers from the room awareness', () => {
		const editor = localDeckEditor(new Y.Doc(), { name: 'Ana', color: '#000' });
		const other = { user: { name: 'Bo', color: '#f00' }, deck: { slide: 's', selection: ['x', 3] } };
		editor.awareness.states.set(42, other);
		editor.awareness.emit('change', [{ added: [42], updated: [], removed: [] }, 'remote']);
		expect(editor.peers).toEqual([{ client: 42, name: 'Bo', color: '#f00', slide: 's', selection: ['x'] }]);
		expect(editor.awareness.getLocalState()?.user).toEqual({ name: 'Ana', color: '#000' });
	});
});
