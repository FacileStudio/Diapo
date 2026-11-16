<script lang="ts">
	let {
		layout = 'default',
		title,
		sub = '',
		bullets = [],
		shown = bullets.length,
		class: extra = ''
	}: {
		/** The scene layout, as the format names it. */
		layout?: 'default' | 'cover' | 'center';
		title: string;
		/** The muted line of a cover or a centered scene. */
		sub?: string;
		bullets?: string[];
		/** How many bullets are revealed. */
		shown?: number;
		class?: string;
	} = $props();
</script>

<div class="ds ds-{layout} {extra}">
	<div class="ds-content">
		{#if title}
			<span class="ds-title">{title}</span>
		{/if}
		{#if sub}<span class="ds-sub">{sub}</span>{/if}
		{#if bullets.length > 0}
			<ul class="ds-list">
				{#each bullets as bullet, index (index)}
					<li class:ds-hidden={index >= shown || !bullet}>{bullet}</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>

<style>
	.ds {
		container-type: inline-size;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		background: var(--color-fc-on-inverse);
		color: var(--color-fc-inverse);
		font-family:
			ui-sans-serif,
			system-ui,
			-apple-system,
			'Segoe UI',
			Roboto,
			'Helvetica Neue',
			Arial,
			sans-serif;
	}
	.ds-content {
		display: flex;
		flex-direction: column;
		height: 100%;
		padding: 5.8cqw 6.7cqw;
		font-size: calc(2.08cqw * var(--ds-zoom, 1.2));
		line-height: 1.4;
	}
	.ds-title {
		margin-bottom: 1.9cqw;
		font-size: calc(3.75cqw * var(--ds-zoom, 1.2));
		font-weight: 650;
		line-height: 1.08;
		letter-spacing: -0.03em;
	}
	.ds-sub {
		color: var(--color-fc-on-inverse-muted);
		filter: brightness(0.72);
	}
	.ds-list {
		margin: 0;
		padding-left: 1.2em;
		list-style: disc;
	}
	.ds-list li {
		margin-bottom: 1.04cqw;
		padding-left: 0.2em;
		transition: opacity 0.35s ease;
	}
	.ds-list li::marker {
		color: var(--color-fc-on-inverse-muted);
	}
	.ds-hidden {
		opacity: 0;
	}
	.ds-cover .ds-content {
		justify-content: flex-end;
	}
	.ds-cover .ds-title {
		font-size: calc(7.08cqw * var(--ds-zoom, 1.2));
	}
	.ds-cover .ds-sub {
		font-size: calc(2.5cqw * var(--ds-zoom, 1.2));
	}
	.ds-center .ds-content {
		align-items: center;
		justify-content: center;
		text-align: center;
	}
	@media (prefers-reduced-motion: reduce) {
		.ds-list li {
			transition: none;
		}
	}
</style>
