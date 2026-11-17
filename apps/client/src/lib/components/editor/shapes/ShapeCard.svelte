<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';

	let { shape }: { shape: Shape<FullType> } = $props();
</script>

<div class="card">
	<strong>{shape.title || shape.url}</strong>
	{#if shape.description}<span>{shape.description}</span>{/if}
	{#if shape.site}<small>{shape.site}</small>{/if}
</div>

<style>
	.card {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: 100%;
		padding: 32px;
		border-radius: 28px;
		background: var(--fill);
	}

	small {
		color: var(--muted);
	}
</style>
