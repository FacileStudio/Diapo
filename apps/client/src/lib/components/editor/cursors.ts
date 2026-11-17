import type { Handle } from '@facile/planche';

const BASE: Record<Handle, number> = { n: 0, ne: 45, e: 90, se: 135, s: 180, sw: 225, w: 270, nw: 315 };
const CURSORS = ['ns-resize', 'nesw-resize', 'ew-resize', 'nwse-resize'] as const;

/** The resize cursor of `handle` on a box turned by `rot` radians, as Toile picks it: the nearest of the four. */
export function handleCursor(handle: Handle, rot = 0): string {
	const deg = (((BASE[handle] + (rot * 180) / Math.PI) % 360) + 360) % 360;
	return CURSORS[Math.round(deg / 45) % 4];
}
