import { icons, keyCaps, type MenuEntry } from '@facile/muse';
import { TEXT_TYPES, type Point, type ShapeType } from '@facile/planche';
import type { TextAlign } from '$lib/editor/actions';
import type { StageController } from '$lib/editor/stage.svelte';
import type { Insertable } from '$lib/editor/types';
import { t } from '$lib/i18n';
import type { AppKey } from '$lib/i18n/messages';
import { tools } from './tools';

/** What the stage's menus act on besides the controller: opening a text box, the files panel, the shortcut list. */
export type StageMenuHooks = {
	edit: (id: string) => void;
	files: () => void;
	help: () => void;
};

const SEPARATOR = { separator: true } as const;

/** The shortcut `keys` (`mod+shift+z`) spelled for this platform: `⌘⇧Z` on Apple, `Ctrl+Shift+Z` elsewhere. */
export function spell(keys: string): string {
	const apple = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
	return keyCaps(keys, apple).join(apple ? '' : '+');
}

function clipEntries(controller: StageController, at: Point | null): MenuEntry[] {
	const { actions } = controller;
	return [
		{ label: t('menu.cut'), shortcut: 'mod+x', icon: tools.cut, onselect: () => void actions.cut() },
		{ label: t('menu.copy'), shortcut: 'mod+c', icon: icons.copy, onselect: () => void actions.copy() },
		{ label: t('menu.paste'), shortcut: 'mod+v', icon: tools.paste, onselect: () => void actions.paste(at ?? undefined) },
		{ label: t('menu.duplicate'), shortcut: 'mod+d', onselect: () => actions.duplicate() }
	];
}

function orderEntries(controller: StageController): MenuEntry[] {
	const rows = [
		['front', 'tool.front', 'mod+shift+]'],
		['forward', 'menu.forward', 'mod+]'],
		['backward', 'menu.backward', 'mod+['],
		['back', 'tool.back', 'mod+shift+[']
	] as const;
	return rows.map(([where, label, shortcut]) => ({
		label: t(label),
		shortcut,
		icon: where === 'front' ? tools.front : where === 'back' ? tools.back : undefined,
		onselect: () => controller.actions.reorder(where)
	}));
}

function arrangeEntries(controller: StageController): MenuEntry[] {
	const { actions } = controller;
	const count = actions.selected.length;
	if (count === 1)
		return [{ label: t('menu.centerSlide'), onselect: () => (actions.align('center'), actions.align('middle')) }];
	const edges = ['left', 'center', 'right', 'top', 'middle', 'bottom'] as const;
	const align = edges.map((edge) => ({ label: t(`arrange.${edge}`), onselect: () => actions.align(edge) }));
	const spread = (['horizontal', 'vertical'] as const).map((axis) => ({
		label: t(`arrange.${axis}`),
		disabled: count < 3,
		onselect: () => actions.distribute(axis)
	}));
	return [{ label: t('menu.alignDistribute'), items: [...align, SEPARATOR, ...spread] }];
}

/** The text alignment submenu: horizontal then vertical, each with its glyph; shortcuts on the horizontal ones. */
function textEntry(apply: (patch: TextAlign) => void): MenuEntry {
	const across = (['left', 'center', 'right', 'justify'] as const).map((align, index) => ({
		label: t(`align.${align}`),
		shortcut: `mod+shift+${'lerj'[index]}`,
		icon: tools[`align-${align}`],
		onselect: () => apply({ align })
	}));
	const down = (['top', 'middle', 'bottom'] as const).map((valign) => ({
		label: t(`valign.${valign}`),
		icon: tools[`valign-${valign}`],
		onselect: () => apply({ valign })
	}));
	return { label: t('menu.textAlign'), icon: tools['align-left'], items: [...across, SEPARATOR, ...down] };
}

/** The menu of a right-click on the selection. */
export function shapeMenu(controller: StageController, at: Point | null, hooks: StageMenuHooks): MenuEntry[] {
	const { actions } = controller;
	const shapes = actions.selected;
	const worded = shapes.some((shape) => TEXT_TYPES.has(shape.type as ShapeType));
	const locked = shapes.every((shape) => shape.locked);
	const lowest = Math.min(...shapes.map((shape) => shape.step ?? 0));
	const single = shapes.length === 1 && worded ? shapes[0].id : null;
	const opener: MenuEntry[] = single ? [{ label: t('menu.edit'), shortcut: 'enter', onselect: () => hooks.edit(single) }] : [];
	return [
		...opener,
		...clipEntries(controller, at),
		SEPARATOR,
		{ label: t('menu.arrange'), icon: tools.front, items: orderEntries(controller) },
		...arrangeEntries(controller),
		...(worded ? [textEntry((patch) => actions.textAlign(patch))] : []),
		SEPARATOR,
		{ label: t(locked ? 'menu.unlock' : 'menu.lock'), icon: tools.lock, onselect: () => actions.lock(!locked) },
		{ label: t('menu.stepLater'), onselect: () => actions.step(1) },
		{ label: t('menu.stepEarlier'), disabled: lowest === 0, onselect: () => actions.step(-1) },
		SEPARATOR,
		{ label: t('menu.delete'), shortcut: 'delete', icon: icons.remove, tone: 'danger', onselect: () => controller.editor.remove(shapes.map((shape) => shape.id)) }
	];
}

/** The menu of a right-click on the empty slide: paste, insert at the point, select all and the shortcut list. */
export function stageMenu(controller: StageController, at: Point | null, hooks: StageMenuHooks): MenuEntry[] {
	const { width, height } = controller.editor.size;
	const point = at ?? { x: width / 2, y: height / 2 };
	const kinds: [Insertable, AppKey, string][] = [
		['text', 'tool.text', icons.text],
		['rect', 'tool.rect', tools.rect],
		['ellipse', 'tool.ellipse', tools.ellipse],
		['diamond', 'tool.diamond', tools.diamond]
	];
	return [
		{ label: t('menu.paste'), shortcut: 'mod+v', icon: tools.paste, onselect: () => void controller.actions.paste(at ?? undefined) },
		SEPARATOR,
		...kinds.map(([kind, label, icon]) => ({
			label: t('menu.insert', { what: t(label) }),
			icon,
			onselect: () => void controller.insertAt(kind, point)
		})),
		{ label: t('menu.media'), icon: tools.image, onselect: hooks.files },
		SEPARATOR,
		{ label: t('menu.selectAll'), shortcut: 'mod+a', disabled: controller.shapes.length === 0, onselect: () => controller.editor.select(controller.shapes.map((shape) => shape.id)) },
		{ label: t('menu.shortcuts'), shortcut: '?', icon: icons.help, onselect: hooks.help }
	];
}
