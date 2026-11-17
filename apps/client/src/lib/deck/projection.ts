import { outline } from './outline';

/** What the present and presenter pages read of a stored source: the stage size and each slide's notes. */
export interface DeckPlan {
	width: number;
	height: number;
	notes: string[];
}

type Fence = { type: string; from: number; to: number };

const FENCE = /^(`{3,})(\w+)(?: |$)/;

/** Whether `source` is a planche projection rather than a legacy scene source. */
export function isPlanche(source: string): boolean {
	return source.replace(/^﻿/, '').startsWith('---\nplanche:');
}

function fences(lines: string[], from: number, to: number): Fence[] {
	const found: Fence[] = [];
	for (let at = from; at < to; at += 1) {
		const open = FENCE.exec(lines[at]);
		if (!open) continue;
		const close = lines.indexOf(open[1], at + 1);
		const end = close < 0 || close > to ? to : close;
		found.push({ type: open[2], from: at + 1, to: end });
		at = end;
	}
	return found;
}

function notesOf(lines: string[], frame: Fence): string {
	const first = fences(lines, frame.from, frame.to)[0];
	return first?.type === 'notes' ? lines.slice(first.from, first.to).join('\n').trim() : '';
}

function planche(source: string): DeckPlan {
	const lines = source.replace(/^﻿/, '').split(/\r?\n/);
	const body = lines.indexOf('---', 1) + 1;
	const size = /^size: (\d+)x(\d+)$/m.exec(lines.slice(0, body).join('\n'));
	const frames = fences(lines, body, lines.length).filter((fence) => fence.type === 'frame');
	const notes = frames.map((frame) => notesOf(lines, frame));
	return { width: Number(size?.[1] ?? 1920), height: Number(size?.[2] ?? 1080), notes };
}

/** Reads either format: a planche projection by its frames in order, a legacy source by its scenes. */
export function readPlan(source: string): DeckPlan {
	if (isPlanche(source)) return planche(source);
	const found = outline(source);
	return { width: found.width, height: found.height, notes: found.scenes.map((scene) => scene.note) };
}
