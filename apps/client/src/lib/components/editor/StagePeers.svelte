<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';
	import type { Peer } from '$lib/editor/types';

	let {
		peers,
		shapes,
		slide,
		scale
	}: { peers: Peer[]; shapes: Shape<FullType>[]; slide: string | null; scale: number } = $props();

	const marks = $derived(
		peers
			.filter((peer) => peer.slide === slide)
			.flatMap((peer) =>
				shapes.filter((shape) => peer.selection.includes(shape.id)).map((shape) => ({ peer, shape }))
			)
	);
</script>

{#each marks as { peer, shape } (`${peer.client}-${shape.id}`)}
	<div
		class="peer"
		style:left="{shape.x}px"
		style:top="{shape.y}px"
		style:width="{shape.w}px"
		style:height="{shape.h}px"
		style:transform={shape.rot ? `rotate(${shape.rot}rad)` : undefined}
		style:outline="{2 / scale}px solid {peer.color}"
	>
		{#if peer.name}
			<span
				class="name"
				style:background={peer.color}
				style:font-size="{12 / scale}px"
				style:padding="{2 / scale}px {6 / scale}px"
				style:border-radius="{4 / scale}px"
			>
				{peer.name}
			</span>
		{/if}
	</div>
{/each}

<style>
	.peer {
		position: absolute;
		pointer-events: none;
		z-index: 2;
	}

	.name {
		position: absolute;
		left: 0;
		bottom: 100%;
		color: #fff;
		font-family: ui-sans-serif, system-ui, sans-serif;
		line-height: 1.3;
		white-space: nowrap;
		transform: translateY(-4px);
	}
</style>
