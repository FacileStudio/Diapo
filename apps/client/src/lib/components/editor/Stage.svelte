<script lang="ts">
	import type { Handle, Point } from '@facile/planche';
	import { StageController, type Target } from '$lib/editor/stage.svelte';
	import type { DeckEditor } from '$lib/editor/types';
	import StagePeers from './StagePeers.svelte';
	import StageSelection from './StageSelection.svelte';
	import StageShape from './StageShape.svelte';

	let {
		editor,
		src,
		class: className = ''
	}: {
		/** The open deck. */
		editor: DeckEditor;
		/** The URL of a deck asset such as `assets/chart.png`. */
		src: (asset: string) => string;
		class?: string;
	} = $props();

	const controller = $derived(new StageController(editor));
	let width = $state(0);
	let height = $state(0);
	let slide = $state<HTMLDivElement>();

	const size = $derived(editor.size);
	const scale = $derived(width && height ? Math.min(width / size.width, height / size.height) : 0);
	const shapes = $derived(controller.shapes.map((shape) => controller.view(shape)));
	const selected = $derived(shapes.filter((shape) => editor.selection.includes(shape.id)));

	$effect(() => {
		controller.scale = scale || 1;
	});

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

	function focusable(node: HTMLDivElement) {
		node.tabIndex = 0;
		const up = () => controller.up();
		const move = (event: PointerEvent) => controller.move(point(event), event);
		const events = Object.entries({
			pointerdown: onpointerdown,
			pointermove: move,
			pointerup: up,
			pointercancel: up,
			dblclick: ondblclick,
			keydown: onkeydown
		}) as [string, EventListener][];
		for (const [name, handler] of events) node.addEventListener(name, handler);
		return () => events.forEach(([name, handler]) => node.removeEventListener(name, handler));
	}

	function onpointerdown(event: PointerEvent) {
		if (event.button !== 0 || !slide) return;
		controller.down(targetOf(event.target as Element), point(event), event);
		if (!controller.dragging) return;
		event.preventDefault();
		slide.focus();
		slide.setPointerCapture(event.pointerId);
	}

	function ondblclick(event: MouseEvent) {
		const target = targetOf(event.target as Element);
		if (target.kind === 'shape') controller.open(target.id);
	}

	function onkeydown(event: KeyboardEvent) {
		if (controller.key(event)) event.preventDefault();
		if (event.key === 'Escape') slide?.focus();
	}
</script>

<div bind:clientWidth={width} bind:clientHeight={height} class="relative size-full overflow-hidden {className}">
	<div
		bind:this={slide}
		class="stage-slide"
		role="application"
		{@attach focusable}
		style:width="{size.width}px"
		style:height="{size.height}px"
		style:left="{(width - size.width * scale) / 2}px"
		style:top="{(height - size.height * scale) / 2}px"
		style:transform="scale({scale})"
	>
		{#each shapes as shape (shape.id)}
			<StageShape {shape} {editor} {src} />
		{/each}
		<StagePeers peers={editor.peers} {shapes} slide={editor.current} scale={scale || 1} />
		{#if !editor.editing}
			<StageSelection shapes={selected} scale={scale || 1} />
		{/if}
		{#each controller.guides.x as x (x)}
			<div class="guide" style:left="{x}px" style:top="0" style:width="{1 / (scale || 1)}px" style:height="100%"></div>
		{/each}
		{#each controller.guides.y as y (y)}
			<div class="guide" style:top="{y}px" style:left="0" style:height="{1 / (scale || 1)}px" style:width="100%"></div>
		{/each}
		{#if controller.area}
			<div
				class="area"
				style:left="{controller.area.x}px"
				style:top="{controller.area.y}px"
				style:width="{controller.area.w}px"
				style:height="{controller.area.h}px"
			></div>
		{/if}
	</div>
</div>

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

	.guide {
		position: absolute;
		background: var(--color-fc-accent);
		pointer-events: none;
		z-index: 3;
	}

	.area {
		position: absolute;
		background: color-mix(in srgb, var(--color-fc-accent) 12%, transparent);
		pointer-events: none;
		z-index: 3;
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
