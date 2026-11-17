import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { localDeckEditor } from './deck.svelte';
import { problems } from './testing';

describe('slides', () => {
	it('adds slides left to right in order, each a frame with notes', () => {
		const editor = localDeckEditor();
		const a = editor.addSlide();
		const c = editor.addSlide();
		const b = editor.addSlide(a);
		expect(editor.slides.map((slide) => slide.id)).toEqual([a, b, c]);
		expect(editor.slides.map((slide) => editor.doc.shapes.get(slide.id)?.get('x'))).toEqual([0, 2120, 4240]);
		expect(editor.doc.shapes.get(a)?.get('name')).toBe('');
		expect(editor.current).toBe(b);
		expect(problems(editor.doc)).toEqual([]);
	});

	it('reads the size from meta and orders frames without an order last', () => {
		const ydoc = new Y.Doc();
		ydoc.getMap('meta').set('size', '1280x720');
		const editor = localDeckEditor(ydoc);
		const first = editor.addSlide();
		editor.doc.shapes.get(first)?.delete('order');
		const second = editor.addSlide();
		expect(editor.size).toEqual({ width: 1280, height: 720 });
		expect(editor.slides.map((slide) => slide.id)).toEqual([second, first]);
		expect(editor.doc.shapes.get(first)?.get('x')).toBe(1480);
	});

	it('stacks the shapes of a slide by track, then z, as the compiler does', () => {
		const editor = localDeckEditor();
		editor.addSlide();
		const low = editor.insert('rect');
		const high = editor.insert('rect');
		editor.doc.shapes.get(low)?.set('track', 1);
		expect(editor.slides[0].shapes.map((shape) => shape.id)).toEqual([high, low]);
	});

	it('moves, duplicates and removes slides with their shapes', () => {
		const editor = localDeckEditor();
		const a = editor.addSlide();
		const b = editor.addSlide();
		editor.show(a);
		const box = editor.insert('rect');
		editor.moveSlide(a, null);
		expect(editor.slides.map((slide) => slide.id)).toEqual([b, a]);
		editor.moveSlide(a, b);
		expect(editor.slides.map((slide) => slide.id)).toEqual([a, b]);
		const copy = editor.duplicateSlide(a);
		expect(editor.slides.map((slide) => slide.id)).toEqual([a, copy, b]);
		const copied = editor.slides[1].shapes;
		expect(copied).toHaveLength(1);
		expect(copied[0].id).not.toBe(box);
		expect(editor.notes(copy)).not.toBe(editor.notes(a));
		editor.removeSlide(a);
		expect(editor.slides.map((slide) => slide.id)).toEqual([copy, b]);
		expect(editor.doc.shapes.has(box)).toBe(false);
		expect(editor.current).toBe(copy);
		expect(problems(editor.doc)).toEqual([]);
	});

	it('creates missing notes once, outside the undo history', () => {
		const editor = localDeckEditor();
		const slide = editor.addSlide();
		editor.doc.shapes.get(slide)?.delete('notes');
		editor.undo();
		expect(editor.slides).toEqual([]);
		editor.redo();
		editor.doc.shapes.get(slide)?.delete('notes');
		const notes = editor.notes(slide);
		expect(editor.notes(slide)).toBe(notes);
		expect(notes.toString()).toBe('<paragraph></paragraph>');
		editor.undo();
		expect(editor.slides).toEqual([]);
	});
});
