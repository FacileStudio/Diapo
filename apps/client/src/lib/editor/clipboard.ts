import {
	isShapeType,
	keysBetween,
	richText,
	union,
	type BoardDoc,
	type FullType,
	type Point,
	type Shape
} from '@facile/planche';
import { readSlides } from './read';
import { jsonToRich, richToJson, type RichNode } from './richjson';
import type { Slide } from './types';
import { createShape, emptyRich, updateShape, type Fields } from './write';
import { hasText } from './css';

/** One copied shape: its plain fields, and its rich text as JSON. */
export type ClipShape = { id: string; type: FullType; fields: Fields; text?: RichNode[] };

/** What the clipboard carries between slides, decks and tabs. */
export type Clip = { v: 1; shapes: ClipShape[] };

/** The text a Diapo clipboard payload starts with; anything else on the clipboard is plain text. */
export const CLIP_PREFIX = 'application/x-diapo-shapes;v=1,';

const STEP = 40;
const DROPPED = new Set(['id', 'type', 'text', 'parent', 'z', 'notes', 'order']);

function clipShape(doc: BoardDoc, shape: Shape<FullType>): ClipShape {
	const fields = Object.fromEntries(Object.entries(shape).filter(([key]) => !DROPPED.has(key))) as Fields;
	const map = doc.shapes.get(shape.id);
	const rich = map && richText(map, 'text');
	const text = rich ? richToJson(rich) : shape.text ? plainNodes(shape.text) : undefined;
	return { id: shape.id, type: shape.type, fields, ...(text ? { text } : {}) };
}

function plainNodes(text: string): RichNode[] {
	return text.split('\n').map((line) => ({ type: 'paragraph', content: line ? [{ text: line }] : [] }));
}

/** The clipboard text of shapes `ids`, in paint order; frames are left out. */
export function encodeShapes(doc: BoardDoc, ids: string[]): string {
	const wanted = new Set(ids);
	const shapes = readSlides(doc)
		.flatMap((slide) => slide.shapes)
		.filter((shape) => wanted.has(shape.id));
	const clip: Clip = { v: 1, shapes: shapes.map((shape) => clipShape(doc, shape)) };
	return CLIP_PREFIX + JSON.stringify(clip);
}

/** A clip read back from clipboard text, or null when the text is not one. Types the `full` profile refuses are dropped. */
export function decodeShapes(text: string): Clip | null {
	if (!text.startsWith(CLIP_PREFIX)) return null;
	try {
		const raw = JSON.parse(text.slice(CLIP_PREFIX.length)) as { shapes?: unknown };
		if (!Array.isArray(raw.shapes)) return null;
		const shapes = (raw.shapes as Partial<ClipShape>[]).filter(
			(shape) => shape && typeof shape.id === 'string' && isShapeType(shape.type, 'full') && shape.type !== 'frame'
		) as ClipShape[];
		return {
			v: 1,
			shapes: shapes.map((shape) => ({ ...shape, fields: typeof shape.fields === 'object' ? { ...shape.fields } : {} }))
		};
	} catch {
		return null;
	}
}

/** A clip of one text box holding `text`, centred on a stage of `size`. */
export function textClip(text: string, size: { width: number; height: number }): Clip {
	const [w, h] = [Math.min(960, size.width), 160];
	const fields: Fields = { x: Math.round((size.width - w) / 2), y: Math.round((size.height - h) / 2), w, h };
	return { v: 1, shapes: [{ id: 'text', type: 'text', fields: { ...fields, style: { size: 64 } }, text: plainNodes(text) }] };
}

function offsetOf(clip: Clip, slide: Slide, at: Point | undefined): Point {
	const boxes = clip.shapes.map(({ fields: f }) => ({
		x: Number(f.x) || 0,
		y: Number(f.y) || 0,
		w: Number(f.w) || 0,
		h: Number(f.h) || 0
	}));
	const box = union(boxes);
	if (!box) return { x: 0, y: 0 };
	if (at) return { x: Math.round(at.x - box.x - box.w / 2), y: Math.round(at.y - box.y - box.h / 2) };
	const corners = new Set(slide.shapes.map((shape) => `${shape.x},${shape.y}`));
	let step = 0;
	while (corners.has(`${boxes[0].x + step},${boxes[0].y + step}`) && step < STEP * 50) step += STEP;
	return { x: step, y: step };
}

function rebound(fields: Fields, ids: Map<string, string>): Fields {
	const out: Fields = {};
	for (const key of ['from', 'to'] as const) {
		const binding = fields[key];
		const fresh = binding && typeof binding === 'object' ? ids.get(binding.shape) : undefined;
		out[key] = fresh && binding ? { ...binding, shape: fresh } : undefined;
	}
	return out;
}

/** Pastes `clip` on top of `slide`: centred on `at` when given, else offset until it covers no copy; answers the new ids. */
export function pasteShapes(doc: BoardDoc, slide: Slide, clip: Clip, at?: Point): string[] {
	const shift = offsetOf(clip, slide, at);
	const zs = keysBetween(slide.shapes.at(-1)?.z || null, null, clip.shapes.length);
	const ids = new Map<string, string>();
	doc.transact(() => {
		clip.shapes.forEach((shape, index) => {
			const { x, y } = shape.fields;
			const place = { x: (Number(x) || 0) + shift.x, y: (Number(y) || 0) + shift.y, parent: slide.id, z: zs[index] };
			const fields = { ...shape.fields, ...place, from: undefined, to: undefined };
			const rich = jsonToRich(shape.text) ?? (hasText(shape.type) ? emptyRich() : undefined);
			ids.set(shape.id, createShape(doc, shape.type, fields, rich));
		});
		for (const shape of clip.shapes) {
			const fresh = ids.get(shape.id);
			if (fresh && (shape.fields.from || shape.fields.to)) updateShape(doc, fresh, rebound(shape.fields, ids));
		}
	});
	return [...ids.values()];
}
