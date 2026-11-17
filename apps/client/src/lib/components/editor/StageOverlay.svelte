<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';
	import type { StageController } from '$lib/editor/stage.svelte';
	import { t } from '$lib/i18n';
	import StageOutline from './StageOutline.svelte';
	import StagePeers from './StagePeers.svelte';
	import StageSelection from './StageSelection.svelte';

	let {
		controller,
		shapes,
		scale
	}: {
		controller: StageController;
		/** The shapes of the slide on show, drag preview applied. */
		shapes: Shape<FullType>[];
		/** Screen pixels per frame pixel. */
		scale: number;
	} = $props();

	const editor = $derived(controller.editor);
	const selected = $derived(shapes.filter((shape) => editor.selection.includes(shape.id)));
	const hovered = $derived(
		controller.dragging || !controller.hover || editor.selection.includes(controller.hover)
			? null
			: (shapes.find((shape) => shape.id === controller.hover) ?? null)
	);
	const area = $derived(controller.area);
</script>

<StagePeers peers={editor.peers} {shapes} slide={editor.current} {scale} />
{#if hovered}
	<StageOutline shape={hovered} {scale} />
{/if}
<StageSelection shapes={selected} {scale} handles={!editor.editing} />
{#each controller.guides.x as x (x)}
	<div class="guide top-0 h-full w-[1.5px]" style:left="{x * scale - 0.75}px"></div>
{/each}
{#each controller.guides.y as y (y)}
	<div class="guide left-0 h-[1.5px] w-full" style:top="{y * scale - 0.75}px"></div>
{/each}
{#if area}
	<div
		class="area"
		style:left="{area.x * scale}px"
		style:top="{area.y * scale}px"
		style:width="{area.w * scale}px"
		style:height="{area.h * scale}px"
		data-testid="marquee"
	></div>
{/if}
{#if editor.synced && shapes.length === 0}
	<p class="hint" data-testid="stage-hint">{t('stage.empty')}</p>
{/if}

<style>
	.guide {
		position: absolute;
		background: var(--color-fc-info);
		pointer-events: none;
		z-index: 20;
	}

	.area {
		position: absolute;
		background: color-mix(in srgb, var(--color-fc-surface) 50%, transparent);
		box-shadow: 0 0 0 1.5px var(--color-fc-info);
		pointer-events: none;
		z-index: 20;
	}

	.hint {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px;
		color: #6b6b6b;
		font-size: 14px;
		text-align: center;
		pointer-events: none;
	}
</style>
