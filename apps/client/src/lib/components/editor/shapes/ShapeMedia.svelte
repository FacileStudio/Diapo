<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';

	let { shape, src }: { shape: Shape<FullType>; src: (asset: string) => string } = $props();

	const url = $derived(shape.asset ? src(shape.asset) : '');
	const fit = $derived(shape.fit ?? 'contain');
</script>

{#if url && shape.type === 'video'}
	<video src={url} muted playsinline preload="metadata" class="block size-full" style:object-fit={fit}></video>
{:else if url}
	<img src={url} alt="" draggable="false" class="block size-full" style:object-fit={fit} />
{/if}
