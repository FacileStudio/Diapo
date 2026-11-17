import {
	center,
	intersects,
	resizeBox,
	rotate,
	rotatedBox,
	scaleInto,
	snapBox,
	union,
	type Box,
	type FullType,
	type Handle,
	type Point,
	type Shape
} from '@facile/planche';
import type { Fields } from './write';

/** The guide lines a snapped drag shows, in frame pixels. */
export type Guides = { x: number[]; y: number[] };

/** What a drag writes: fields per shape id, and the guides to draw. */
export type DragResult = { changes: Map<string, Fields>; guides: Guides };

type Item = Shape<FullType>;

/** The box around `shapes`, rotation applied, or null for none. */
export function boundsOf(shapes: Item[]): Box | null {
	return union(shapes.map((shape) => rotatedBox(shape)));
}

/** Moves the unlocked `shapes` by the pointer delta, snapped to `targets` within `threshold` pixels. */
export function moveDrag(shapes: Item[], delta: Point, targets: Box[], threshold: number): DragResult {
	const free = shapes.filter((shape) => !shape.locked);
	const box = boundsOf(free);
	if (!box) return { changes: new Map(), guides: { x: [], y: [] } };
	const moved = { ...box, x: box.x + delta.x, y: box.y + delta.y };
	const snap = threshold > 0 ? snapBox(moved, targets, threshold) : { dx: 0, dy: 0, guides: { x: [], y: [] } };
	const [dx, dy] = [Math.round(delta.x + snap.dx), Math.round(delta.y + snap.dy)];
	const changes = new Map(free.map((shape) => [shape.id, { x: shape.x + dx, y: shape.y + dy } as Fields]));
	return { changes, guides: snap.guides };
}

function resizeOne(shape: Item, handle: Handle, delta: Point, keep: boolean): Fields {
	const local = rotate(delta, { x: 0, y: 0 }, -shape.rot);
	const next = resizeBox(shape, handle, local.x, local.y, keep);
	const pin = (box: Box) => {
		const fixed = { x: box.x + (handle.includes('w') ? box.w : 0), y: box.y + (handle.includes('n') ? box.h : 0) };
		return rotate(fixed, center(box), shape.rot);
	};
	const [before, after] = [pin(shape), pin(next)];
	return { x: next.x + before.x - after.x, y: next.y + before.y - after.y, w: next.w, h: next.h };
}

/** Resizes the unlocked `shapes` by dragging `handle`: one box keeps its turn, several scale inside their bounds. */
export function resizeDrag(shapes: Item[], handle: Handle, delta: Point, keep: boolean): Map<string, Fields> {
	const free = shapes.filter((shape) => !shape.locked);
	if (free.length === 1 && free[0].type !== 'line' && free[0].type !== 'arrow' && free[0].type !== 'ink')
		return new Map([[free[0].id, resizeOne(free[0], handle, delta, keep)]]);
	const from = boundsOf(free);
	if (!from) return new Map();
	const to = resizeBox(from, handle, delta.x, delta.y, keep);
	return new Map(free.map((shape) => [shape.id, scaleInto(shape as Shape, from, to) as Fields]));
}

/** The turn of `shape` when the rotate handle, grabbed at `start`, reaches `at`; `step` snaps to 15 degrees. */
export function rotateDrag(shape: Item, start: Point, at: Point, step: boolean): number {
	const c = center(shape);
	const angle = Math.atan2(at.y - c.y, at.x - c.x) - Math.atan2(start.y - c.y, start.x - c.x) + shape.rot;
	const unit = Math.PI / 12;
	return step ? Math.round(angle / unit) * unit : angle;
}

/** The box between two pointer positions. */
export function areaOf(a: Point, b: Point): Box {
	return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) };
}

/** The ids of the shapes a marquee touches. */
export function marquee(shapes: Item[], area: Box): string[] {
	return shapes.filter((shape) => intersects(area, rotatedBox(shape))).map((shape) => shape.id);
}
