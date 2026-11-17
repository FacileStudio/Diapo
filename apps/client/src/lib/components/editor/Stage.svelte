<script lang="ts">
	import { ContextMenu, type MenuEntry } from '@facile/muse';
	import type { Handle, Point } from '@facile/planche';
	import { tick } from 'svelte';
	import { listenClipboard } from '$lib/editor/clipevents';
	import { StageController, type Target } from '$lib/editor/stage.svelte';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import { shapeMenu, stageMenu } from './menus';
	import StageOverlay from './StageOverlay.svelte';
	import StageShape from './StageShape.svelte';

	let {
		editor,
		src,
		onfiles = () => undefined,
		onhelp = () => undefined,
		class: className = ''
	}: {
		/** The open deck. */
		editor: DeckEditor;
		/** The URL of a deck asset such as `assets/chart.png`. */
		src: (asset: string) => string;
		/** Opens the files panel, to place an image or a video. */
		onfiles?: () => void;
		/** Opens the list of keyboard shortcuts. */
		onhelp?: () => void;
		class?: string;
	} = $props();

	const controller = $derived(new StageController(editor));
	let width = $state(0);
	let height = $state(0);
	let slide = $state<HTMLDivElement>();
	let menuAt: Point | null = null;

	const size = $derived(editor.size);
	const scale = $derived(width && height ? Math.min(width / size.width, height / size.height) : 0);
	const shapes = $derived(controller.shapes.map((shape) => controller.view(shape)));
	const left = $derived((width - size.width * scale) / 2);
	const top = $derived((height - size.height * scale) / 2);
	const writable = $derived(editor.synced && editor.stopped === null);

	$effect(() => {
		controller.scale = scale || 1;
		controller.onhelp = onhelp;
	});

	$effect(() => {
		const id = editor.editing;
		if (id) void tick().then(() => focusText(id));
	});

	function focusText(id: string) {
		const prose = slide?.querySelector<HTMLElement>(`[data-shape="${CSS.escape(id)}"] .ProseMirror`);
		if (prose && !prose.contains(document.activeElement)) prose.focus();
	}

	function point(event: PointerEvent | MouseEvent): Point {
		const rect = slide?.getBoundingClientRect();
		if (!rect || !scale) return { x: 0, y: 0 };
		return { x: (event.clientX - rect.left) / scale, y: (event.clientY - rect.top) / scale };
	}

	function targetOf(element: Element): Target {
		const handle = element.closest<HTMLElement>('[data-handle]')?.dataset.handle;
		if (handle) return { kind: 'handle', handle: handle as Handle };
		if (element.closest('[data-rotate]')) return { kind: 'rotate' };
		const id = element.closest<HTMLElement>('[data-shape]')?.dataset.shape;
		return id ? { kind: 'shape', id } : { kind: 'empty' };
	}

	function entries(element: Element): MenuEntry[] {
		if (!writable) return [];
		const target = targetOf(element);
		const hooks = { edit: (id: string) => controller.open(id), files: onfiles, help: onhelp };
		if (target.kind === 'shape' && !editor.selection.includes(target.id)) editor.select([target.id]);
		if (target.kind === 'empty' && menuAt) editor.select([]);
		if (editor.editing) return [];
		return editor.selection.length ? shapeMenu(controller, menuAt, hooks) : stageMenu(controller, menuAt, hooks);
	}

	function focusable(node: HTMLDivElement) {
		node.tabIndex = 0;
		const events = Object.entries({
			pointerdown: onpointerdown,
			pointermove: (event: PointerEvent) => {
				controller.hover = (targetOf(event.target as Element) as { id?: string }).id ?? null;
				controller.move(point(event), event);
			},
			pointerleave: () => (controller.hover = null),
			pointerup: () => controller.up(),
			pointercancel: () => controller.up(),
			dblclick: ondblclick,
			keydown: onkeydown,
			contextmenu: (event: MouseEvent) => (menuAt = event.clientX || event.clientY ? point(event) : null)
		}) as [string, EventListener][];
		for (const [name, handler] of events) node.addEventListener(name, handler);
		const stop = listenClipboard(node, controller.actions, () => writable);
		return () => {
			stop();
			events.forEach(([name, handler]) => node.removeEventListener(name, handler));
		};
	}

	function onpointerdown(event: PointerEvent) {
		const node = event.currentTarget as HTMLElement;
		menuAt = point(event);
		if (event.button !== 0 || !writable) return;
		controller.down(targetOf(event.target as Element), point(event), event);
		if (!controller.dragging) return;
		event.preventDefault();
		node.focus();
		node.setPointerCapture(event.pointerId);
	}

	function ondblclick(event: MouseEvent) {
		const target = targetOf(event.target as Element);
		if (target.kind === 'shape') controller.open(target.id);
		else if (target.kind === 'empty' && writable) controller.insertAt('text', point(event));
	}

	function onkeydown(event: KeyboardEvent) {
		menuAt = null;
		if (controller.key(event)) event.preventDefault();
		if (event.key === 'Escape') (event.currentTarget as HTMLElement).focus();
	}
</script>

<ContextMenu items={entries} label={t('menu.stage')} class="size-full">
	<div
		bind:clientWidth={width}
		bind:clientHeight={height}
		class="relative size-full touch-none overflow-hidden rounded-fc-lg outline-none focus-visible:outline-2
			focus-visible:-outline-offset-2 focus-visible:outline-fc-ring {className}"
		role="application"
		aria-label={t('stage.label')}
		aria-roledescription={t('stage.role')}
		data-testid="stage"
		{@attach focusable}
	>
		<div
			bind:this={slide}
			class="stage-slide"
			style:width="{size.width}px"
			style:height="{size.height}px"
			style:left="{left}px"
			style:top="{top}px"
			style:transform="scale({scale})"
		>
			{#each shapes as shape (shape.id)}
				<StageShape {shape} {editor} {src} />
			{/each}
		</div>
		<div
			class="pointer-events-none absolute"
			style:left="{left}px"
			style:top="{top}px"
			style:width="{size.width * scale}px"
			style:height="{size.height * scale}px"
		>
			<StageOverlay {controller} {shapes} scale={scale || 1} />
		</div>
	</div>
</ContextMenu>

<style>
	.stage-slide {
		position: absolute;
		transform-origin: 0 0;
		overflow: hidden;
		outline: none;
		background: #fff;
		color: #1a1a1a;
		--fill: #f4f4f4;
		--muted: #6b6b6b;
		font-family: ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
		font-size: 40px;
		line-height: 1.4;
		box-shadow: 0 8px 40px rgba(0, 0, 0, 0.12);
		touch-action: none;
	}

	.stage-slide :global(.stage-text),
	.stage-slide :global(.stage-text .text-fc-fg) {
		color: inherit;
		gap: 0;
	}

	.stage-slide :global(.stage-text .encre-prose) {
		min-height: 0;
		line-height: inherit;
		overflow-wrap: break-word;
	}

	.stage-slide :global(.stage-text :is(p, ul, ol, blockquote, pre, table, h1, h2, h3, h4)) {
		margin: 0 0 36px;
	}

	.stage-slide :global(.stage-text .encre-prose > :last-child) {
		margin-bottom: 0;
	}

	.stage-slide :global(.stage-text :is(h1, h2, h3, h4)) {
		font-weight: 650;
		line-height: 1.08;
		letter-spacing: -0.03em;
	}

	.stage-slide :global(.stage-text h1) {
		font-size: 104px;
	}

	.stage-slide :global(.stage-text h2) {
		font-size: 72px;
	}

	.stage-slide :global(.stage-text h3) {
		font-size: 52px;
	}
</style>
