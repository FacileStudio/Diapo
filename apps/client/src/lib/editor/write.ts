import * as Y from 'yjs';
import {
	LIMITS,
	allowedKey,
	clampNumber,
	cleanBinding,
	cleanStyle,
	isShapeType,
	newId,
	type BoardDoc,
	type FullType,
	type Shape
} from '@facile/planche';

/** The fields a caller may write on a shape: every plain key but `id`, `type` and the rich `text`. */
export type Fields = Omit<Partial<Shape<FullType>>, 'id' | 'type' | 'text'>;

const SKIPPED = new Set(['id', 'type', 'text', 'notes']);
const FITS = new Set(['cover', 'contain', 'fill']);

const whole = (max: number) => (value: unknown) => Math.min(max, Math.max(0, Math.round(Number(value) || 0)));
const unit = (value: unknown) => Math.max(0, Math.min(1, Number(value) || 0));
const flag = (value: unknown) => (typeof value === 'boolean' ? value : undefined);
const text = (max: number) => (value: unknown) => (typeof value === 'string' ? value.slice(0, max) : undefined);
const number = (value: unknown) => clampNumber(Number(value));

const CLEANERS: Record<string, (value: unknown) => unknown> = {
	x: number,
	y: number,
	w: number,
	h: number,
	rot: number,
	start: number,
	duration: number,
	media_in: number,
	step: whole(1000),
	track: whole(64),
	opacity: unit,
	volume: unit,
	locked: flag,
	fixed_duration: flag,
	enter: text(64),
	layout: text(64),
	order: text(64),
	html: text(LIMITS.htmlChars),
	fit: (value) => (typeof value === 'string' && FITS.has(value) ? value : undefined),
	points: (value) => (Array.isArray(value) ? value.map((n) => clampNumber(Number(n))) : undefined),
	style: (value) => cleanStyle(value, 'full'),
	from: cleanBinding,
	to: cleanBinding
};

function clean(key: string, value: unknown): unknown {
	const cleaner = CLEANERS[key];
	return cleaner ? cleaner(value) : text(10_000)(value);
}

/** Writes `fields` into a shape's map: keys the `full` profile refuses are dropped, `undefined` deletes. */
export function writeFields(map: Y.Map<unknown>, type: FullType, fields: Fields): void {
	for (const [key, value] of Object.entries(fields)) {
		if (SKIPPED.has(key) || !allowedKey(type, key, 'full')) continue;
		const kept = value == null ? undefined : clean(key, value);
		if (kept === undefined) map.delete(key);
		else if (!map.doc || map.get(key) !== kept) map.set(key, kept);
	}
}

/** Updates shape `id` with `fields` in one local transaction; an unknown shape is ignored. */
export function updateShape(doc: BoardDoc, id: string, fields: Fields): void {
	const map = doc.shapes.get(id);
	const type = map?.get('type');
	if (!map || !isShapeType(type, 'full')) return;
	doc.transact(() => writeFields(map, type, fields));
}

/** A fresh rich text in encre's collaborative layout: one empty paragraph. */
export function emptyRich(): Y.XmlFragment {
	const fragment = new Y.XmlFragment();
	fragment.insert(0, [new Y.XmlElement('paragraph')]);
	return fragment;
}

/** Creates a shape of `type` in one local transaction and returns its id; `text` becomes its rich text. */
export function createShape(doc: BoardDoc, type: FullType, fields: Fields, text?: Y.XmlFragment): string {
	if (doc.shapes.size >= LIMITS.shapes) throw new Error('board-full');
	const id = newId();
	const map = new Y.Map<unknown>();
	map.set('type', type);
	writeFields(map, type, { rot: 0, style: {}, ...fields });
	if (text && allowedKey(type, 'text', 'full')) map.set('text', text);
	doc.transact(() => doc.shapes.set(id, map));
	return id;
}

/** Deletes shapes and clears the bindings and parents that named them, in one local transaction. */
export function removeShapes(doc: BoardDoc, ids: Iterable<string>): void {
	const gone = new Set(ids);
	doc.transact(() => {
		gone.forEach((id) => doc.shapes.delete(id));
		doc.shapes.forEach((map) => {
			for (const key of ['from', 'to'] as const) {
				const binding = map.get(key) as { shape?: string } | undefined;
				if (binding?.shape && gone.has(binding.shape)) map.delete(key);
			}
			if (gone.has(map.get('parent') as string)) map.delete('parent');
		});
	});
}
