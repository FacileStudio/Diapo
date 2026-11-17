import { flushSync, mount, unmount } from 'svelte';
import { describe, expect, it } from 'vitest';
import { localDeckEditor } from '$lib/editor/deck.svelte';
import Stage from './Stage.svelte';

globalThis.ResizeObserver ??= class {
	observe(): void {}
	unobserve(): void {}
	disconnect(): void {}
} as unknown as typeof ResizeObserver;

describe('Stage', () => {
	it('draws the shown slide: boxes, sandboxed html, media, steps and editable text', async () => {
		const editor = localDeckEditor();
		const box = editor.insert('rect', { x: 10, y: 20, w: 300, h: 200 });
		editor.update(box, { step: 2, style: { fill: '#f00' } });
		const html = editor.insert('image', undefined, 'assets/a.png');
		editor.doc.transact(() => editor.doc.shapes.get(html)?.set('type', 'html'));
		editor.update(html, { html: '<b>hi</b>' });
		editor.insert('video', undefined, 'assets/b.mp4');
		const text = editor.insert('text');
		const target = document.createElement('div');
		document.body.append(target);
		const stage = mount(Stage, { target, props: { editor, src: (name: string) => `/x/${name}` } });
		flushSync();
		const rect = target.querySelector<HTMLElement>(`[data-shape="${box}"]`);
		expect(rect?.style).toMatchObject({ left: '10px', top: '20px', width: '300px', height: '200px' });
		expect(rect?.style.background).toBe('rgb(255, 0, 0)');
		expect(rect?.querySelector('.step')?.textContent).toBe('2');
		const frame = target.querySelector('iframe');
		expect(frame?.getAttribute('sandbox')).toBe('allow-scripts');
		expect(frame?.getAttribute('srcdoc')).toBe('<b>hi</b>');
		expect(target.querySelector('video')?.getAttribute('src')).toBe('/x/assets/b.mp4');
		await new Promise((resolve) => setTimeout(resolve, 0));
		const prose = target.querySelector(`[data-shape="${text}"] .ProseMirror`);
		expect(prose?.getAttribute('contenteditable')).toBe('true');
		editor.edit(null);
		flushSync();
		expect(prose?.getAttribute('contenteditable')).toBe('false');
		void unmount(stage);
		target.remove();
	});
});
