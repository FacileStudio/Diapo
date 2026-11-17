<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';
	import type { Peer } from '$lib/editor/types';
	import StageOutline from './StageOutline.svelte';

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
	<StageOutline {shape} {scale} color={peer.color} />
	{#if peer.name}
		<span
			class="pointer-events-none absolute z-10 rounded-fc-sm px-1.5 py-0.5 text-fc-xs whitespace-nowrap text-black"
			style:left="{shape.x * scale}px"
			style:top="{shape.y * scale}px"
			style:transform="translateY(calc(-100% - 4px))"
			style:background={peer.color}
		>
			{peer.name}
		</span>
	{/if}
{/each}
