import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { plainText } from './text';

describe('plainText', () => {
	it('reads one line per block and drops the formatting', () => {
		const doc = new Y.Doc();
		const fragment = doc.getXmlFragment('t');
		const blocks = ['heading', 'paragraph'].map((name) => new Y.XmlElement(name));
		fragment.insert(0, blocks);
		const title = new Y.XmlText();
		blocks[0].insert(0, [title]);
		title.insert(0, 'Hello ');
		title.insert(6, 'world', { bold: true });
		const body = new Y.XmlText();
		blocks[1].insert(0, [body]);
		body.insert(0, 'Second');
		expect(plainText(fragment)).toBe('Hello world\nSecond');
		expect(plainText(undefined)).toBe('');
	});
});
