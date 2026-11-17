import { keyBetween, type BoardDoc } from '@facile/planche';
import { SLIDE_GAP, readSize, readSlides } from './read';
import { copyShape } from './copy';
import { createShape, emptyRich, removeShapes, writeFields } from './write';

function layout(doc: BoardDoc): void {
	const { width } = readSize(doc);
	readSlides(doc).forEach((slide, index) => {
		const map = doc.shapes.get(slide.id);
		if (map) writeFields(map, 'frame', { x: index * (width + SLIDE_GAP), y: 0 });
	});
}

function neighbours(doc: BoardDoc, after: string | undefined): [string | null, string | null] {
	const slides = readSlides(doc);
	const index = after === undefined ? slides.length - 1 : slides.findIndex((slide) => slide.id === after);
	if (index < 0) return [slides.at(-1)?.order ?? null, null];
	return [slides[index].order || null, slides[index + 1]?.order || null];
}

/** Adds an empty slide after `after` (the end by default) and returns its id. */
export function addSlide(doc: BoardDoc, after?: string): string {
	const { width, height } = readSize(doc);
	const [a, b] = neighbours(doc, after);
	let id = '';
	doc.transact(() => {
		id = createShape(doc, 'frame', { x: 0, y: 0, w: width, h: height, name: '', order: keyBetween(a, b) });
		doc.shapes.get(id)?.set('notes', emptyRich());
		layout(doc);
	});
	return id;
}

/** Copies slide `id`, its notes and its shapes, right after it, and returns the copy's id. */
export function duplicateSlide(doc: BoardDoc, id: string): string {
	const slide = readSlides(doc).find((item) => item.id === id);
	if (!slide) return '';
	const [a, b] = neighbours(doc, id);
	let copy = '';
	doc.transact(() => {
		copy = copyShape(doc, id, { order: keyBetween(a, b) });
		const ids = new Map<string, string>();
		for (const shape of slide.shapes) ids.set(shape.id, copyShape(doc, shape.id, { parent: copy }));
		for (const fresh of ids.values()) rebind(doc, fresh, ids);
		layout(doc);
	});
	return copy;
}

function rebind(doc: BoardDoc, id: string, ids: Map<string, string>): void {
	const map = doc.shapes.get(id);
	for (const key of ['from', 'to'] as const) {
		const binding = map?.get(key) as { shape: string; anchor: [number, number] } | undefined;
		const fresh = binding && ids.get(binding.shape);
		if (map && binding && fresh) map.set(key, { ...binding, shape: fresh });
	}
}

/** Removes slides with every shape they hold. */
export function removeSlides(doc: BoardDoc, ids: string[]): void {
	const frames = new Set(ids);
	const gone = [...frames];
	doc.shapes.forEach((map, id) => {
		if (frames.has(map.get('parent') as string)) gone.push(id);
	});
	doc.transact(() => {
		removeShapes(doc, gone);
		layout(doc);
	});
}

/** Moves slide `id` before slide `before`, or to the end when `before` is null. */
export function moveSlide(doc: BoardDoc, id: string, before: string | null): void {
	if (id === before) return;
	const slides = readSlides(doc).filter((slide) => slide.id !== id);
	const index = before === null ? slides.length : slides.findIndex((slide) => slide.id === before);
	if (index < 0) return;
	const order = keyBetween(slides[index - 1]?.order || null, slides[index]?.order || null);
	doc.transact(() => {
		const map = doc.shapes.get(id);
		if (map) writeFields(map, 'frame', { order });
		layout(doc);
	});
}
