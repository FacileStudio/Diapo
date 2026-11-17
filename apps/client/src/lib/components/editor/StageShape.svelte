<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';
	import { boxStyle, hasText, lookStyle } from '$lib/editor/css';
	import { strokeOf } from '$lib/editor/strokes';
	import type { DeckEditor } from '$lib/editor/types';
	import ShapeCard from './shapes/ShapeCard.svelte';
	import ShapeHtml from './shapes/ShapeHtml.svelte';
	import ShapeMedia from './shapes/ShapeMedia.svelte';
	import ShapeStroke from './shapes/ShapeStroke.svelte';
	import ShapeText from './shapes/ShapeText.svelte';

	let {
		shape,
		editor,
		src
	}: { shape: Shape<FullType>; editor: DeckEditor; src: (asset: string) => string } = $props();

	const stroke = $derived(
		shape.type === 'line' || shape.type === 'arrow' || shape.type === 'ink' ? strokeOf(shape) : null
	);
	const text = $derived(hasText(shape.type));
	const style = $derived(
		[boxStyle(shape, stroke ?? undefined), text ? lookStyle(shape) : ''].filter(Boolean).join(';')
	);
	const editing = $derived(editor.editing === shape.id);
</script>

{#if shape.type !== 'audio'}
	<div
		data-shape={shape.id}
		class="pl pl-{shape.type}"
		class:editing
		class:cursor-text={editing}
		{style}
	>
		{#if stroke}
			<ShapeStroke {stroke} />
		{:else if text}
			<ShapeText {shape} {editor} />
		{:else if shape.type === 'image' || shape.type === 'video'}
			<ShapeMedia {shape} {src} />
		{:else if shape.type === 'html'}
			<ShapeHtml html={shape.html ?? ''} />
		{:else if shape.type === 'card'}
			<ShapeCard {shape} />
		{/if}
		{#if (shape.step ?? 0) > 0}
			<span class="step" aria-hidden="true">{shape.step}</span>
		{/if}
	</div>
{/if}

<style>
	.pl {
		position: absolute;
		margin: 0;
		box-sizing: border-box;
		user-select: none;
	}

	.pl.editing {
		user-select: text;
		z-index: 1;
	}

	.pl-text,
	.pl-sticky,
	.pl-rect,
	.pl-ellipse,
	.pl-diamond {
		overflow-wrap: break-word;
	}

	.pl-rect,
	.pl-ellipse,
	.pl-diamond {
		display: flex;
		flex-direction: column;
		justify-content: center;
		text-align: center;
		padding: 24px;
	}

	.pl-sticky {
		padding: 32px;
		color: #1a1a1a;
		box-shadow: 0 12px 32px rgba(0, 0, 0, 0.14);
	}

	.step {
		position: absolute;
		top: 8px;
		right: 8px;
		min-width: 40px;
		height: 40px;
		padding: 0 10px;
		border-radius: 20px;
		background: var(--color-fc-accent);
		color: var(--color-fc-accent-fg);
		font: 600 24px/40px ui-sans-serif, system-ui, sans-serif;
		text-align: center;
		pointer-events: none;
	}
</style>
