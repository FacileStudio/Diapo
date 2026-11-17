import { compareZ, readShape, type BoardDoc, type FullType, type Shape } from '@facile/planche';
import type { Slide } from './types';

/** The stage size when `meta.size` is missing or unreadable. */
export const DEFAULT_SIZE = { width: 1920, height: 1080 };

/** The gap between two slides laid out left to right, in frame pixels. */
export const SLIDE_GAP = 200;

/** The stage size from `meta.size` (`"<w>x<h>"`), else 1920x1080. */
export function readSize(doc: BoardDoc): { width: number; height: number } {
	const match = /^(\d+)x(\d+)$/.exec(String(doc.meta.get('size') ?? ''));
	const width = Number(match?.[1]);
	const height = Number(match?.[2]);
	if (!width || !height) return { ...DEFAULT_SIZE };
	return { width, height };
}

/** Every shape of the document under the `full` profile, unknown types left out. */
export function readAll(doc: BoardDoc): Shape<FullType>[] {
	const out: Shape<FullType>[] = [];
	doc.shapes.forEach((map, id) => {
		const shape = readShape(id, map, 'full');
		if (shape) out.push(shape);
	});
	return out;
}

/** Orders frames as slides: by `order`, a frame without one last, then by id. */
export function compareOrder(a: Shape<FullType>, b: Shape<FullType>): number {
	if (Boolean(a.order) !== Boolean(b.order)) return a.order ? -1 : 1;
	return compareZ({ z: a.order ?? '', id: a.id }, { z: b.order ?? '', id: b.id });
}

/** Orders the shapes of one slide as the compiler stacks them: by `track` (0 without one), then by `z`. */
export function compareStack(a: Shape<FullType>, b: Shape<FullType>): number {
	return (a.track ?? 0) - (b.track ?? 0) || compareZ(a, b);
}

/** The slides of the document: top-level frames by `order`, each with its children by `track`, then `z`. */
export function readSlides(doc: BoardDoc): Slide[] {
	const all = readAll(doc);
	const frames = all.filter((shape) => shape.type === 'frame' && !shape.parent).sort(compareOrder);
	const children = new Map<string, Shape<FullType>[]>(frames.map((frame) => [frame.id, []]));
	for (const shape of all) if (shape.type !== 'frame' && shape.parent) children.get(shape.parent)?.push(shape);
	return frames.map((frame) => ({
		id: frame.id,
		name: frame.name ?? '',
		order: frame.order ?? '',
		shapes: (children.get(frame.id) ?? []).sort(compareStack)
	}));
}
