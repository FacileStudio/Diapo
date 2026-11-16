import type { ExportFormat } from '$lib/types';

const KNOWN_KEY = 'diapo.exports.ready';
const ASKED_KEY = 'diapo.exports.asked';

/** How long a `503` is believed before the format is asked again, in ms. */
export const ASK_AGAIN_AFTER = 6 * 60 * 60 * 1000;

/** When each format was last answered `503`, in ms since the epoch. */
export function asked(): Partial<Record<ExportFormat, number>> {
	try {
		return JSON.parse(localStorage.getItem(ASKED_KEY) ?? '{}') as Partial<Record<ExportFormat, number>>;
	} catch {
		return {};
	}
}

/** Remembers that the API just answered `503` for `format`. */
export function refused(format: ExportFormat): void {
	try {
		localStorage.setItem(ASKED_KEY, JSON.stringify({ ...asked(), [format]: Date.now() }));
	} catch {
		return;
	}
}

/** The formats this browser has seen the API produce. HTML and PowerPoint shipped first. */
export function known(): ExportFormat[] {
	try {
		return JSON.parse(localStorage.getItem(KNOWN_KEY) ?? '["html","pptx"]') as ExportFormat[];
	} catch {
		return ['html', 'pptx'];
	}
}

/** Remembers that the API produced `format`. */
export function learn(format: ExportFormat): void {
	try {
		localStorage.setItem(KNOWN_KEY, JSON.stringify([...new Set([...known(), format])]));
	} catch {
		return;
	}
}
