import { rotatedBox, union, type Box, type FullType, type Shape } from '@facile/planche';
import type { Fields } from './write';

/** An edge or centre line shapes line up on. */
export type Edge = 'left' | 'center' | 'right' | 'top' | 'middle' | 'bottom';

/** The direction shapes spread along with equal gaps. */
export type Axis = 'horizontal' | 'vertical';

type Item = Shape<FullType>;

const HORIZONTAL = new Set<Edge>(['left', 'center', 'right']);

function target(edge: Edge, bounds: Box): number {
	if (edge === 'left') return bounds.x;
	if (edge === 'right') return bounds.x + bounds.w;
	if (edge === 'center') return bounds.x + bounds.w / 2;
	if (edge === 'top') return bounds.y;
	if (edge === 'bottom') return bounds.y + bounds.h;
	return bounds.y + bounds.h / 2;
}

function shift(edge: Edge, box: Box, line: number): number {
	const near = HORIZONTAL.has(edge) ? box.x : box.y;
	const span = HORIZONTAL.has(edge) ? box.w : box.h;
	if (edge === 'left' || edge === 'top') return line - near;
	if (edge === 'right' || edge === 'bottom') return line - near - span;
	return line - near - span / 2;
}

/** Lines the unlocked `shapes` up on `edge` of their bounds, or of `stage` for a lone shape; answers the moves. */
export function alignShapes(shapes: Item[], edge: Edge, stage: Box): Map<string, Fields> {
	const free = shapes.filter((shape) => !shape.locked);
	const bounds = shapes.length === 1 ? stage : union(shapes.map((shape) => rotatedBox(shape)));
	const out = new Map<string, Fields>();
	if (!bounds) return out;
	const line = target(edge, bounds);
	for (const shape of free) {
		const delta = Math.round(shift(edge, rotatedBox(shape), line));
		if (delta === 0) continue;
		out.set(shape.id, HORIZONTAL.has(edge) ? { x: shape.x + delta } : { y: shape.y + delta });
	}
	return out;
}

/** Spreads three or more unlocked `shapes` along `axis` so the gaps between them are equal; answers the moves. */
export function distributeShapes(shapes: Item[], axis: Axis): Map<string, Fields> {
	const out = new Map<string, Fields>();
	const free = shapes.filter((shape) => !shape.locked);
	if (free.length < 3) return out;
	const [pos, size] = axis === 'horizontal' ? (['x', 'w'] as const) : (['y', 'h'] as const);
	const boxes = free.map((shape) => ({ shape, box: rotatedBox(shape) })).sort((a, b) => a.box[pos] - b.box[pos]);
	const first = boxes[0].box;
	const last = boxes[boxes.length - 1].box;
	const used = boxes.reduce((sum, item) => sum + item.box[size], 0);
	const gap = (last[pos] + last[size] - first[pos] - used) / (boxes.length - 1);
	let at = first[pos];
	for (const { shape, box } of boxes) {
		const delta = Math.round(at - box[pos]);
		if (delta !== 0) out.set(shape.id, { [pos]: shape[pos] + delta });
		at += box[size] + gap;
	}
	return out;
}
