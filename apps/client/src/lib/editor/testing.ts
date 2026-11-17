import * as Y from 'yjs';
import { allowedKey, isShapeType, type BoardDoc } from '@facile/planche';

/** Every problem that makes `doc` refused by a `full` planche server: unknown types or keys, bad text. */
export function problems(doc: BoardDoc): string[] {
	const out: string[] = [];
	for (const name of doc.ydoc.share.keys()) if (name !== 'meta' && name !== 'shapes') out.push(`root ${name}`);
	doc.shapes.forEach((map, id) => {
		const type = map.get('type');
		if (!isShapeType(type, 'full')) return out.push(`${id}: type ${String(type)}`);
		for (const [key, value] of map.entries()) {
			if (!allowedKey(type, key, 'full')) out.push(`${id}: key ${key} on ${type}`);
			const rich = key === 'notes' || (key === 'text' && !(value instanceof Y.Text));
			if (rich && !(value instanceof Y.XmlFragment && value.length > 0)) out.push(`${id}: ${key} is not rich`);
		}
		if (typeof map.get('x') !== 'number' || typeof map.get('y') !== 'number') out.push(`${id}: no position`);
	});
	return out;
}
