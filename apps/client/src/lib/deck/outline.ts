import { blocks } from './split';

/** What the client reads of one scene, without compiling it. Lines count from 1. */
export interface SceneOutline {
	index: number;
	/** The line the scene opens on: line 1, or its `---` separator. */
	from: number;
	/** The last line of the scene. */
	to: number;
	layout: string;
	steps: number;
	/** The text of the first heading, or the first line of text. */
	title: string;
	/** The presenter note: the trailing HTML comment of the body. */
	note: string;
}

/** A deck source as the client reads it: the stage size and one outline per scene. */
export interface Outline {
	width: number;
	height: number;
	scenes: SceneOutline[];
}

function note(body: string): string {
	const text = body.trimEnd();
	const start = text.lastIndexOf('<!--');
	if (start < 0 || !text.endsWith('-->')) return '';
	if (start > 0 && text[start - 1] !== '\n') return '';
	const inner = text.slice(start + 4, -3);
	return inner.includes('-->') ? '' : inner.trim();
}

function steps(front: Record<string, string>, body: string): number {
	const declared = Number.parseInt(front.steps ?? '', 10);
	if (Number.isFinite(declared)) return declared;
	const used = [...body.matchAll(/\{step=(\d+)\}|data-step="(\d+)"/g)].map((hit) => Number(hit[1] ?? hit[2]));
	return Math.max(0, ...used);
}

function title(body: string[]): string {
	const text = body.map((line) => line.trim()).filter((line) => line && !line.startsWith('<'));
	const heading = text.find((line) => line.startsWith('#')) ?? text[0] ?? '';
	return heading.replace(/^#+\s*/, '').replace(/\s*\{step=\d+\}$/, '');
}

/** Reads `source` the way `docs/format.md` splits it: scenes, their layout, step count and presenter note. */
export function outline(source: string): Outline {
	const lines = source.replace(/^﻿/, '').split(/\r?\n/);
	const all = blocks(lines);
	const last = all[all.length - 1];
	const trailing = all.length > 1 && Object.keys(last.front).length === 0 && last.body.every((line) => !line.trim());
	const kept = trailing ? all.slice(0, -1) : all;
	const size = /^(\d+)x(\d+)$/.exec(kept[0]?.front.size ?? '');
	const scenes = kept.map((block, index) => {
		const body = block.body.join('\n');
		return {
			index,
			from: block.from,
			to: block.to,
			layout: block.front.layout || 'default',
			steps: steps(block.front, body),
			title: title(block.body),
			note: note(body)
		};
	});
	return { width: Number(size?.[1] ?? 1920), height: Number(size?.[2] ?? 1080), scenes };
}

/** The index of the scene that holds `line` (from 1). */
export function sceneAt(scenes: SceneOutline[], line: number): number {
	let found = 0;
	for (const scene of scenes) if (scene.from <= line) found = scene.index;
	return found;
}
