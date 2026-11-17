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

const NUMERIC = new Set(['x', 'y', 'w', 'h', 'rot', 'start', 'duration', 'media_in']);
const SKIPPED = new Set(['id', 'type', 'text', 'notes']);

function clean(key: string, value: unknown): unknown {
	if (NUMERIC.has(key)) return clampNumber(value as number);
	if (key === 'step' || key === 'track') return Math.max(0, Math.round(Number(value) || 0));
	if (key === 'opacity' || key === 'volume') return Math.max(0, Math.min(1, Number(value) || 0));
	if (key === 'points') return Array.isArray(value) ? value.map((n) => clampNumber(Number(n))) : undefined;
	if (key === 'style') return cleanStyle(value, 'full');
	if (key === 'from' || key === 'to') return cleanBinding(value);
	if (key === 'locked' || key === 'fixed_duration') return typeof value === 'boolean' ? value : undefined;
	if (key === 'html' && typeof value === 'string') return value.slice(0, LIMITS.htmlChars);
	if (typeof value === 'string') return value.slice(0, 10_000);
	return value;
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
