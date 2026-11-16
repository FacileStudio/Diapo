/** One scene as the splitter finds it: its lines (from 1), its frontmatter keys and its body lines. */
export type Block = { from: number; to: number; front: Record<string, string>; body: string[] };

const SEPARATOR = /^---[ \t]*$/;
const KEY_LINE = /^([a-z0-9_]+):(?: (.*))?$/;
const FENCE = /^ {0,3}(`{3,}|~{3,})/;

function scalar(raw: string): string {
	const text = raw.trim();
	const quoted = /^(["'])(.*)\1$/.exec(text);
	if (quoted) return quoted[2];
	return text.replace(/\s+#.*$/, '');
}

function fenceAfter(open: string | null, line: string): string | null {
	const marker = FENCE.exec(line)?.[1];
	if (!marker) return open;
	if (open === null) return marker;
	return marker[0] === open[0] && marker.length >= open.length && line.trim() === marker ? null : open;
}

/** Reads the frontmatter that starts at `start`; answers its keys and the index of the line after its closing `---`. */
function frontmatter(lines: string[], start: number): { front: Record<string, string>; next: number } {
	const front: Record<string, string> = {};
	let at = start;
	while (at < lines.length && !SEPARATOR.test(lines[at])) {
		const found = KEY_LINE.exec(lines[at]);
		if (found) front[found[1]] = scalar(found[2] ?? '');
		at += 1;
	}
	return { front, next: at + 1 };
}

function opens(lines: string[], separator: number, head: boolean): boolean {
	const next = lines[separator + 1];
	if (next === undefined) return false;
	return head || SEPARATOR.test(next) || KEY_LINE.test(next);
}

/** Splits `lines` into scenes the way `docs/format.md` does: `---` outside a code fence, then an optional frontmatter. */
export function blocks(lines: string[]): Block[] {
	const found: Block[] = [];
	let current: Block = { from: 1, to: 1, front: {}, body: [] };
	let fence: string | null = null;
	let at = 0;
	while (at < lines.length) {
		const line = lines[at];
		if (fence === null && SEPARATOR.test(line)) {
			if (at > 0) found.push({ ...current, to: at });
			current = { from: at + 1, to: at + 1, front: {}, body: [] };
			if (opens(lines, at, at === 0)) {
				const read = frontmatter(lines, at + 1);
				current.front = read.front;
				at = read.next;
				continue;
			}
		} else {
			fence = fenceAfter(fence, line);
			current.body.push(line);
		}
		at += 1;
	}
	found.push({ ...current, to: Math.max(lines.length, current.from) });
	return found;
}
