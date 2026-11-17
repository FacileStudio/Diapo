<script lang="ts">
	import { CORNERS, rotatedBox, union, type FullType, type Handle, type Shape } from '@facile/planche';
	import { handleCursor } from './cursors';
	import StageOutline from './StageOutline.svelte';

	let {
		shapes,
		scale,
		handles = true
	}: {
		/** The selected shapes, in frame pixels. */
		shapes: Shape<FullType>[];
		/** Screen pixels per frame pixel. */
		scale: number;
		/** False while a text box has the keyboard: the outline stays, the knobs go. */
		handles?: boolean;
	} = $props();

	const POS: Record<Handle, [number, number]> = {
		nw: [0, 0],
		n: [0.5, 0],
		ne: [1, 0],
		e: [1, 0.5],
		se: [1, 1],
		s: [0.5, 1],
		sw: [0, 1],
		w: [0, 0.5]
	};

	const EDGES = [
		{ handle: 'n', place: '-top-1.5 right-2 left-2 h-3' },
		{ handle: 's', place: 'right-2 -bottom-1.5 left-2 h-3' },
		{ handle: 'w', place: 'top-2 bottom-2 -left-1.5 w-3' },
		{ handle: 'e', place: 'top-2 -right-1.5 bottom-2 w-3' }
	] as const;

	const single = $derived(shapes.length === 1 ? shapes[0] : null);
	const box = $derived(single ?? union(shapes.map((shape) => rotatedBox(shape))));
	const rot = $derived(single?.rot ?? 0);
	const movable = $derived(handles && shapes.some((shape) => !shape.locked));
</script>

{#each shapes as shape (shape.id)}
	<StageOutline {shape} {scale} />
{/each}
{#if box && movable}
	<div
		class="pointer-events-none absolute top-0 left-0 z-10 origin-center"
		style:width="{box.w * scale}px"
		style:height="{box.h * scale}px"
		style:transform="translate({box.x * scale}px, {box.y * scale}px) rotate({rot}rad)"
		data-testid="selection"
	>
		{#each EDGES as edge (edge.handle)}
			<span
				class="pointer-events-auto absolute touch-none {edge.place}"
				style:cursor={handleCursor(edge.handle, rot)}
				data-handle={edge.handle}
			></span>
		{/each}
		{#each CORNERS as handle (handle)}
			<span
				class="handle pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center"
				style:left="{POS[handle][0] * 100}%"
				style:top="{POS[handle][1] * 100}%"
				style:cursor={handleCursor(handle, rot)}
				data-handle={handle}
			>
				<span class="size-3 rounded-[3px] bg-fc-info shadow-sm"></span>
			</span>
		{/each}
		{#if single}
			<span
				class="handle group pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 cursor-grab touch-none items-center justify-center"
				style:left="50%"
				style:top="-22px"
				data-rotate
			>
				<span
					class="size-3 rounded-fc-pill bg-fc-info shadow-sm transition-transform group-hover:scale-125 motion-reduce:transition-none"
				></span>
			</span>
		{/if}
	</div>
{/if}

<style>
	.handle {
		width: 24px;
		height: 24px;
	}

	@media (pointer: coarse) {
		.handle {
			width: 44px;
			height: 44px;
		}
	}
</style>
