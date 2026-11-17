import * as Y from 'yjs';

/** The plain text of a rich text fragment, one line per block: what a thumbnail can show. */
export function plainText(node: Y.XmlFragment | Y.XmlElement | Y.XmlText | undefined): string {
	if (!node) return '';
	if (node instanceof Y.XmlText) {
		return node
			.toDelta()
			.map((part: { insert?: unknown }) => (typeof part.insert === 'string' ? part.insert : ''))
			.join('');
	}
	const children = node.toArray().filter((child) => !(child instanceof Y.XmlHook));
	const parts = children.map((child) => plainText(child as Y.XmlElement | Y.XmlText));
	return parts.join(node instanceof Y.XmlElement && node.nodeName === 'paragraph' ? '' : '\n');
}
