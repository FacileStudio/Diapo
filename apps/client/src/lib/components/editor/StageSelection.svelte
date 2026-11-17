<script lang="ts">
	import { HANDLES, rotatedBox, union, type FullType, type Handle, type Shape } from '@facile/planche';

	let { shapes, scale }: { shapes: Shape<FullType>[]; scale: number } = $props();

	const POSITIONS: Record<Handle, [number, number]> = {
		nw: [0, 0],
		n: [50, 0],
		ne: [100, 0],
		e: [100, 50],
		se: [100, 100],
		s: [50, 100],
		sw: [0, 100],
		w: [0, 50]
	};

	const CURSORS: Record<Handle, string> = {
		nw: 'nwse-resize',
		se: 'nwse-resize',
		ne: 'nesw-resize',
		sw: 'nesw-resize',
		n: 'ns-resize',
		s: 'ns-resize',
		e: 'ew-resize',
		w: 'ew-resize'
	};

	const single = $derived(shapes.length === 1 ? shapes[0] : null);
	const frame = $derived(single ?? { ...(union(shapes.map((shape) => rotatedBox(shape))) ?? { x: 0, y: 0, w: 0, h: 0 }), rot: 0 });
	const movable = $derived(shapes.some((shape) => !shape.locked));
	const line = $derived(2 / scale);
	const knob = $derived(12 / scale);
</script>

{#if shapes.length}
	<div
		class="selection"
		style:left="{frame.x}px"
		style:top="{frame.y}px"
		style:width="{frame.w}px"
		style:height="{frame.h}px"
		style:transform={frame.rot ? `rotate(${frame.rot}rad)` : undefined}
		style:outline-width="{line}px"
	>
		{#if movable}
			{#each HANDLES as handle (handle)}
				<span
					data-handle={handle}
					class="knob"
					style:left="{POSITIONS[handle][0]}%"
					style:top="{POSITIONS[handle][1]}%"
					style:width="{knob}px"
					style:height="{knob}px"
					style:cursor={CURSORS[handle]}
				></span>
			{/each}
			{#if single}
				<span
					data-rotate
					class="knob rotate"
					style:left="50%"
					style:top="{-28 / scale}px"
					style:width="{knob}px"
					style:height="{knob}px"
				></span>
			{/if}
		{/if}
	</div>
	{#if !single}
		{#each shapes as shape (shape.id)}
			<div
				class="member"
				style:left="{shape.x}px"
				style:top="{shape.y}px"
				style:width="{shape.w}px"
				style:height="{shape.h}px"
				style:transform={shape.rot ? `rotate(${shape.rot}rad)` : undefined}
				style:outline-width="{line / 2}px"
			></div>
		{/each}
	{/if}
{/if}

<style>
	.selection,
	.member {
		position: absolute;
		pointer-events: none;
		outline-style: solid;
		outline-color: var(--color-fc-accent);
		z-index: 2;
	}

	.knob {
		position: absolute;
		pointer-events: auto;
		transform: translate(-50%, -50%);
		border-radius: 9999px;
		background: var(--color-fc-accent);
		box-shadow: 0 0 0 2px #fff;
	}

	.rotate {
		cursor: grab;
	}
</style>
