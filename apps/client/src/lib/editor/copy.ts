import * as Y from 'yjs';
import { LIMITS, newId, type BoardDoc } from '@facile/planche';
import type { Fields } from './write';

function cloned(value: unknown): unknown {
	if (value instanceof Y.XmlFragment || value instanceof Y.Text) return value.clone();
	if (value && typeof value === 'object') return structuredClone(value);
	return value;
}

/** Copies shape `id` under a fresh id, rich and plain text included, with `fields` on top; answers the new id. */
export function copyShape(doc: BoardDoc, id: string, fields: Fields): string {
	const source = doc.shapes.get(id);
	if (!source) return '';
	if (doc.shapes.size >= LIMITS.shapes) throw new Error('board-full');
	const fresh = newId();
	const map = new Y.Map<unknown>();
	for (const [key, value] of source.entries()) map.set(key, cloned(value));
	for (const [key, value] of Object.entries(fields)) if (value !== undefined) map.set(key, value);
	doc.transact(() => doc.shapes.set(fresh, map));
	return fresh;
}
