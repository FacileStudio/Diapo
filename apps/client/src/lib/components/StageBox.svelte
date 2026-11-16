<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		ratio,
		class: extra = '',
		children
	}: {
		/** The stage's width over its height. */
		ratio: number;
		class?: string;
		children: Snippet;
	} = $props();
</script>

<div class="flex size-full min-h-0 items-center justify-center [container-type:size]">
	<div
		class="max-h-full w-full overflow-hidden {extra}"
		style="aspect-ratio: {ratio}; max-width: calc(100cqh * {ratio})"
	>
		{@render children()}
	</div>
</div>
