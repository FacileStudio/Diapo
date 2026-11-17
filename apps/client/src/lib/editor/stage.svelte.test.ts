import { describe, expect, it } from 'vitest';
import type { FullType, Shape } from '@facile/planche';
import { boxStyle, lookStyle } from './css';
import { localDeckEditor } from './deck.svelte';
import { stageCommand } from './keys';
import { resizeDrag } from './pointer';
import { StageController } from './stage.svelte';
import { strokeOf } from './strokes';

const base: Shape<FullType> = { id: 'a', type: 'rect', x: 10, y: 20, w: 100, h: 50, rot: 0, z: 'V', style: {} };
const press = (key: string, more: Partial<KeyboardEvent> = {}) =>
	({ key, shiftKey: false, metaKey: false, ctrlKey: false, altKey: false, ...more }) as KeyboardEvent;

describe('stage', () => {
	it('writes the compiler css for boxes, fills, shapes and strokes', () => {
		const shape = { ...base, type: 'ellipse', rot: 0.5, opacity: 0.4, style: { fill: '#f00', stroke: '#000', size: 30 } };
		expect(boxStyle(shape as Shape<FullType>)).toBe('left:10px;top:20px;width:100px;height:50px;transform:rotate(0.5rad);opacity:0.4');
		expect(lookStyle(shape as Shape<FullType>)).toBe(
			'background:#f00;border-radius:50%;outline:2px solid #000;outline-offset:0;font-size:30px'
		);
		expect(lookStyle({ ...base, type: 'text', style: { fill: '#00f', stroke: '#000' } })).toBe('color:#00f');
		expect(lookStyle({ ...base, type: 'sticky' })).toBe('background:#fde68a');
		const arrow = strokeOf({ ...base, type: 'arrow', w: 0, h: 0, points: [0, 0, 100, 0] });
		expect(arrow).toMatchObject({ w: 100, h: 1, path: 'M0 0 L100 0', head: '100,0 78.94,11.51 78.94,-11.51', width: 4 });
		expect(strokeOf({ ...base, type: 'ink', points: [0, 0, 1, 5, 5, 1] }).head).toBe('');
	});

	it('maps keys to commands, and only Escape while typing', () => {
		expect(stageCommand(press('ArrowLeft', { shiftKey: true }), false)).toEqual({ kind: 'nudge', dx: -10, dy: 0 });
		expect(stageCommand(press('z', { metaKey: true, shiftKey: true }), false)).toEqual({ kind: 'redo' });
		expect(stageCommand(press('d', { ctrlKey: true }), false)).toEqual({ kind: 'duplicate' });
		expect(stageCommand(press('Backspace'), false)).toEqual({ kind: 'remove' });
		expect(stageCommand(press('Backspace'), true)).toBeNull();
		expect(stageCommand(press('Escape'), true)).toEqual({ kind: 'leave' });
	});

	it('resizes a turned box around its fixed corner and scales several', () => {
		const one = resizeDrag([base], 'se', { x: 20, y: 10 }, false).get('a');
		expect(one).toEqual({ x: 10, y: 20, w: 120, h: 60 });
		const turned = resizeDrag([{ ...base, rot: Math.PI }], 'se', { x: -20, y: 0 }, false).get('a');
		expect(turned?.w).toBeCloseTo(120);
		expect(turned?.x).toBeCloseTo(-10);
		const two = resizeDrag([base, { ...base, id: 'b', x: 110 }], 'e', { x: 200, y: 0 }, false);
		expect(two.get('b')).toMatchObject({ x: 210, w: 200 });
		expect(resizeDrag([{ ...base, locked: true }], 'se', { x: 5, y: 5 }, false).size).toBe(0);
	});

	it('moves with snapping, selects by marquee and commits on release', () => {
		const editor = localDeckEditor();
		const a = editor.insert('rect', { x: 100, y: 100, w: 200, h: 100 });
		const b = editor.insert('rect', { x: 600, y: 600, w: 200, h: 100 });
		const stage = new StageController(editor);
		stage.down({ kind: 'shape', id: a }, { x: 150, y: 150 }, { shiftKey: false, altKey: false });
		stage.move({ x: 552, y: 150 }, { shiftKey: false, altKey: false });
		expect(stage.preview.get(a)).toEqual({ x: 500, y: 100 });
		expect(stage.guides.x).toContain(600);
		expect(editor.slides[0].shapes[0].x).toBe(100);
		stage.up();
		expect(editor.slides[0].shapes[0]).toMatchObject({ x: 500, y: 100 });
		stage.down({ kind: 'empty' }, { x: 0, y: 0 }, { shiftKey: false, altKey: false });
		stage.move({ x: 700, y: 700 }, { shiftKey: false, altKey: false });
		expect(editor.selection).toEqual([a, b]);
		stage.up();
		expect(stage.area).toBeNull();
	});

	it('runs keyboard commands on the selection', () => {
		const editor = localDeckEditor();
		const a = editor.insert('rect', { x: 0, y: 0 });
		const stage = new StageController(editor);
		stage.key(press('ArrowRight', { shiftKey: true }));
		expect(editor.slides[0].shapes[0].x).toBe(10);
		stage.key(press('d', { ctrlKey: true }));
		expect(editor.selection).not.toEqual([a]);
		expect(editor.slides[0].shapes).toHaveLength(2);
		stage.key(press('Enter'));
		expect(editor.editing).toBe(editor.selection[0]);
		stage.key(press('Escape'));
		expect(editor.editing).toBeNull();
		stage.key(press('Delete'));
		expect(editor.slides[0].shapes).toHaveLength(1);
		expect(stage.key(press('z', { metaKey: true }))).toBe(true);
		expect(editor.canRedo).toBe(true);
	});
});
