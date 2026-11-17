import type { MenuAction, MenuEntry } from '@facile/muse';
import { describe, expect, it, vi } from 'vitest';
import { alignShapes, distributeShapes } from '$lib/editor/arrange';
import { localDeckEditor } from '$lib/editor/deck.svelte';
import { stageCommand } from '$lib/editor/keys';
import { StageController } from '$lib/editor/stage.svelte';
import { t } from '$lib/i18n';
import { shapeMenu, stageMenu } from './menus';

const hooks = () => ({ edit: vi.fn(), files: vi.fn(), help: vi.fn() });

function find(entries: MenuEntry[], label: string): MenuAction | undefined {
	for (const entry of entries) {
		if (!('label' in entry)) continue;
		if (entry.label === label) return entry;
		const inner = entry.items && find(entry.items, label);
		if (inner) return inner;
	}
	return undefined;
}

function pick(entries: MenuEntry[], label: string): MenuAction {
	const found = find(entries, label);
	if (!found) throw new Error(`no entry ${label}`);
	return found;
}

const rows = (entries: MenuEntry[]) => entries.filter((entry) => 'label' in entry).length;

describe('menus', () => {
	it('runs the shape menu on the selection: duplicate, order, text alignment, lock, steps, delete', () => {
		const editor = localDeckEditor();
		const controller = new StageController(editor);
		const [a, b] = [editor.insert('text'), editor.insert('rect')];
		editor.select([a]);
		const on = hooks();
		const menu = () => shapeMenu(controller, null, on);
		pick(menu(), t('menu.edit')).onselect?.();
		expect(on.edit).toHaveBeenCalledWith(a);
		pick(menu(), t('menu.duplicate')).onselect?.();
		const copy = editor.selection[0];
		expect(editor.slides[0].shapes.map((shape) => shape.id)).toEqual([a, b, copy]);
		pick(menu(), t('tool.back')).onselect?.();
		expect(editor.slides[0].shapes[0].id).toBe(copy);
		pick(menu(), t('align.justify')).onselect?.();
		pick(menu(), t('valign.bottom')).onselect?.();
		pick(menu(), t('menu.stepLater')).onselect?.();
		pick(menu(), t('menu.lock')).onselect?.();
		expect(editor.slides[0].shapes[0]).toMatchObject({ step: 1, locked: true, style: { align: 'justify', valign: 'bottom' } });
		expect(pick(menu(), t('menu.unlock'))).toBeTruthy();
		expect(pick(menu(), t('menu.textAlign')).items).toHaveLength(8);
		expect(pick(menu(), t('align.justify')).shortcut).toBe('mod+shift+j');
		expect(rows(menu())).toBeLessThanOrEqual(12);
		pick(menu(), t('menu.delete')).onselect?.();
		expect(editor.slides[0].shapes.map((shape) => shape.id)).toEqual([a, b]);
	});

	it('offers arrangement for several shapes and inserting at the point on the empty slide', () => {
		const editor = localDeckEditor();
		const controller = new StageController(editor);
		const on = hooks();
		pick(stageMenu(controller, { x: 100, y: 100 }, on), t('menu.insert', { what: t('tool.rect') })).onselect?.();
		expect(editor.slides[0].shapes[0]).toMatchObject({ type: 'rect', x: 0, y: 0, w: 480, h: 320 });
		editor.insert('ellipse', { x: 600, y: 500 });
		editor.select(editor.slides[0].shapes.map((shape) => shape.id));
		const menu = shapeMenu(controller, null, on);
		expect(rows(menu)).toBeLessThanOrEqual(12);
		expect(pick(menu, t('menu.alignDistribute')).items).toHaveLength(9);
		expect(pick(menu, t('arrange.horizontal')).disabled).toBe(true);
		pick(menu, t('arrange.top')).onselect?.();
		expect(editor.slides[0].shapes.map((shape) => shape.y)).toEqual([0, 0]);
		pick(stageMenu(controller, null, on), t('menu.shortcuts')).onselect?.();
		expect(on.help).toHaveBeenCalled();
	});

	it('aligns to the bounds or the slide, distributes evenly, and maps the new shortcuts', () => {
		const box = (id: string, x: number, w: number) => ({ id, type: 'rect' as const, x, y: 0, w, h: 10, rot: 0, z: '', style: {} });
		const stage = { x: 0, y: 0, w: 1000, h: 500 };
		expect(alignShapes([box('a', 10, 100)], 'center', stage).get('a')).toEqual({ x: 450 });
		expect([...alignShapes([box('a', 10, 100), box('b', 300, 50)], 'right', stage)]).toEqual([['a', { x: 250 }]]);
		const spread = distributeShapes([box('a', 0, 100), box('b', 150, 100), box('c', 400, 100)], 'horizontal');
		expect(spread.get('b')).toEqual({ x: 200 });
		const press = (key: string, shiftKey = false) => ({ key, shiftKey, metaKey: false, ctrlKey: true, altKey: false });
		expect(stageCommand(press('}', true), false)).toEqual({ kind: 'reorder', where: 'front' });
		expect(stageCommand(press('[', false), false)).toEqual({ kind: 'reorder', where: 'backward' });
		expect(stageCommand(press('J', true), true)).toEqual({ kind: 'align', align: 'justify' });
		expect(stageCommand(press('d'), true)).toBeNull();
	});
});
