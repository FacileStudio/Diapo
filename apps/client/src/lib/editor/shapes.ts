import { keyBetween, type BoardDoc, type Box, type FullType } from '@facile/planche';
import { copyShape } from './copy';
import { readSize, readSlides } from './read';
import type { Insertable, Slide } from './types';
import { createShape, emptyRich, writeFields, type Fields } from './write';

/** Where a shape moves in its slide's paint order. */
export type Reorder = 'front' | 'back' | 'forward' | 'backward';

/** The default box of each insertable shape, in frame pixels. */
export const SIZES: Record<Insertable, [number, number]> = {
	text: [960, 160],
	rect: [480, 320],
	ellipse: [360, 360],
	diamond: [360, 360],
	image: [960, 540],
	video: [960, 540]
};

const RICH = new Set<Insertable>(['text', 'rect', 'ellipse', 'diamond']);

const DEFAULTS: Partial<Record<Insertable, Fields>> = {
	text: { style: { size: 64 } },
	rect: { style: { fill: '#e9e9e9' } },
	ellipse: { style: { fill: '#e9e9e9' } },
	diamond: { style: { fill: '#e9e9e9' } }
};

function topZ(slide: Slide): string | null {
	return slide.shapes.at(-1)?.z || null;
}

/** Inserts a `kind` shape on top of `slide`, centred unless `at` says otherwise, and answers its id. */
export function insertShape(doc: BoardDoc, slide: Slide, kind: Insertable, at: Partial<Box> = {}, asset?: string): string {
	const { width, height } = readSize(doc);
	const [w, h] = [at.w ?? SIZES[kind][0], at.h ?? SIZES[kind][1]];
	const box = { x: at.x ?? Math.round((width - w) / 2), y: at.y ?? Math.round((height - h) / 2), w, h };
	const media = kind === 'image' || kind === 'video' ? { asset, fit: 'contain' as const } : {};
	const fields: Fields = { ...DEFAULTS[kind], ...box, ...media, parent: slide.id, z: keyBetween(topZ(slide), null) };
	return createShape(doc, kind as FullType, fields, RICH.has(kind) ? emptyRich() : undefined);
}

function slideOf(doc: BoardDoc, id: string): Slide | undefined {
	return readSlides(doc).find((slide) => slide.shapes.some((shape) => shape.id === id));
}

function newZ(siblings: string[], index: number, where: Reorder): string | null {
	const last = siblings.length - 1;
	if (where === 'front') return index === last ? null : keyBetween(siblings[last] || null, null);
	if (where === 'back') return index === 0 ? null : keyBetween(null, siblings[0] || null);
	if (where === 'forward')
		return index === last ? null : keyBetween(siblings[index + 1] || null, siblings[index + 2] || null);
	return index === 0 ? null : keyBetween(siblings[index - 2] || null, siblings[index - 1] || null);
}

/** Moves shape `id` in its slide's paint order. */
export function reorderShape(doc: BoardDoc, id: string, where: Reorder): void {
	const shapes = slideOf(doc, id)?.shapes ?? [];
	const index = shapes.findIndex((shape) => shape.id === id);
	const z = index < 0 ? null : newZ(shapes.map((shape) => shape.z), index, where);
	const map = doc.shapes.get(id);
	if (z !== null && map) doc.transact(() => writeFields(map, map.get('type') as FullType, { z }));
}

/** Copies shapes inside their slide, offset by 40 pixels and on top, and answers the new ids. */
export function duplicateShapes(doc: BoardDoc, ids: string[]): string[] {
	const out: string[] = [];
	doc.transact(() => {
		for (const id of ids) {
			const slide = slideOf(doc, id);
			const shape = slide?.shapes.find((item) => item.id === id);
			if (!slide || !shape) continue;
			const z = keyBetween(topZ(readSlides(doc).find((item) => item.id === slide.id) ?? slide), null);
			out.push(copyShape(doc, id, { x: shape.x + 40, y: shape.y + 40, z }));
		}
	});
	return out;
}
