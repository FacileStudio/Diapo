import type { FullType, Shape } from '@facile/planche';

/** What a `line`, `arrow` or `ink` shape draws: box size, path, head polygon, colour and width. */
export type Stroke = { w: number; h: number; path: string; head: string; color: string; width: number };

const round = (value: number) => Math.round(value * 100) / 100;

function pointsOf(shape: Shape<FullType>): [number, number][] {
	const stride = shape.type === 'ink' ? 3 : 2;
	const flat = shape.points ?? [];
	const out: [number, number][] = [];
	for (let i = 0; i + stride <= flat.length; i += stride) out.push([round(flat[i]), round(flat[i + 1])]);
	return out;
}

function headOf(at: [number, number][], width: number): string {
	if (at.length < 2) return '';
	const [[x1, y1], [x2, y2]] = at.slice(-2);
	const angle = Math.atan2(y2 - y1, x2 - x1);
	const length = 4 * width + 8;
	const corner = (turn: number) =>
		`${round(x2 - length * Math.cos(angle + turn))},${round(y2 - length * Math.sin(angle + turn))}`;
	return `${x2},${y2} ${corner(-0.5)} ${corner(0.5)}`;
}

/** The stroke of `shape` as the compiler draws it; without `w` and `h` the box reaches the farthest point. */
export function strokeOf(shape: Shape<FullType>): Stroke {
	const at = pointsOf(shape);
	const far = (axis: 0 | 1) => at.reduce((max, point) => Math.max(max, point[axis]), 1);
	const width = shape.style.width ?? 4;
	const pointed = (shape.head ?? (shape.type === 'arrow' ? 'arrow' : 'none')) !== 'none';
	return {
		w: shape.w || far(0),
		h: shape.h || far(1),
		path: at.length ? `M${at.map(([x, y]) => `${x} ${y}`).join(' L')}` : '',
		head: pointed && shape.type !== 'ink' ? headOf(at, width) : '',
		color: shape.style.stroke ?? 'currentColor',
		width
	};
}
