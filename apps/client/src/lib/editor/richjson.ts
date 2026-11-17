import * as Y from 'yjs';
import { schema } from '@facile/encre';

/** One block or run of a rich text as a clipboard carries it: an element with children, or text with marks. */
export type RichNode =
	| { type: string; attrs?: Record<string, Scalar>; content?: RichNode[] }
	| { text: string; marks?: Record<string, unknown> };

type Scalar = string | number | boolean;

const NODES = new Map(schema.nodes.map((node) => [node.name, new Set(node.attrs.map((attr) => attr.name))]));
const MARKS = new Set(schema.marks.map((mark) => mark.name));
const DEPTH = 24;

function scalars(raw: unknown, allowed: Set<string>): Record<string, Scalar> {
	const out: Record<string, Scalar> = {};
	if (!raw || typeof raw !== 'object') return out;
	for (const [key, value] of Object.entries(raw)) {
		const plain = typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean';
		if (allowed.has(key) && plain) out[key] = typeof value === 'string' ? value.slice(0, 10_000) : value;
	}
	return out;
}

function marksOf(raw: unknown): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	if (!raw || typeof raw !== 'object') return out;
	for (const [name, value] of Object.entries(raw)) {
		if (!MARKS.has(name)) continue;
		const attrs = schema.marks.find((mark) => mark.name === name)?.attrs.map((attr) => attr.name) ?? [];
		out[name] = value && typeof value === 'object' ? scalars(value, new Set(attrs)) : true;
	}
	return out;
}

function nodeOf(item: Y.XmlElement | Y.XmlText): RichNode[] {
	if (item instanceof Y.XmlText)
		return (item.toDelta() as { insert?: unknown; attributes?: Record<string, unknown> }[])
			.filter((part) => typeof part.insert === 'string')
			.map((part) => ({ text: part.insert as string, ...(part.attributes ? { marks: part.attributes } : {}) }));
	const content = item.toArray().flatMap((child) => nodeOf(child as Y.XmlElement | Y.XmlText));
	return [{ type: item.nodeName, attrs: item.getAttributes() as Record<string, Scalar>, content }];
}

/** A rich text as plain JSON, for the clipboard. */
export function richToJson(fragment: Y.XmlFragment): RichNode[] {
	return fragment.toArray().flatMap((child) => nodeOf(child as Y.XmlElement | Y.XmlText));
}

function textNode(node: Record<string, unknown>): Y.XmlText | null {
	const value = String(node.text).slice(0, 10_000);
	if (!value) return null;
	const text = new Y.XmlText();
	text.insert(0, value, marksOf(node.marks));
	return text;
}

function elementNode(node: Record<string, unknown>, depth: number): Y.XmlElement | null {
	const allowed = typeof node.type === 'string' ? NODES.get(node.type) : undefined;
	if (!allowed || node.type === 'doc' || node.type === 'text') return null;
	const element = new Y.XmlElement(node.type as string);
	for (const [key, value] of Object.entries(scalars(node.attrs, allowed))) element.setAttribute(key, value as string);
	element.insert(0, build(node.content, depth + 1));
	return element;
}

function build(nodes: unknown, depth: number): (Y.XmlElement | Y.XmlText)[] {
	if (!Array.isArray(nodes) || depth > DEPTH) return [];
	const made = (nodes as unknown[]).map((node) => {
		if (!node || typeof node !== 'object') return null;
		const record = node as Record<string, unknown>;
		return typeof record.text === 'string' ? textNode(record) : elementNode(record, depth);
	});
	return made.filter((node): node is Y.XmlElement | Y.XmlText => node !== null);
}

/** A rich text from clipboard JSON, keeping only encre's node, attribute and mark names; null when nothing is left. */
export function jsonToRich(nodes: unknown): Y.XmlFragment | null {
	const blocks = build(nodes, 0).filter((node) => node instanceof Y.XmlElement);
	if (blocks.length === 0) return null;
	const fragment = new Y.XmlFragment();
	fragment.insert(0, blocks);
	return fragment;
}
