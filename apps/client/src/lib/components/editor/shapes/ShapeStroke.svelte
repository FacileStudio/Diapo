<script lang="ts">
	import type { Stroke } from '$lib/editor/strokes';

	let { stroke }: { stroke: Stroke } = $props();
</script>

<svg
	width="100%"
	height="100%"
	fill={stroke.color}
	stroke={stroke.color}
	stroke-width={stroke.width}
	stroke-linecap="round"
	stroke-linejoin="round"
	style="overflow:visible"
	aria-hidden="true"
>
	{#if stroke.path}<path d={stroke.path} fill="none" />{/if}
	{#if stroke.head}<polygon points={stroke.head} stroke="none" />{/if}
</svg>
