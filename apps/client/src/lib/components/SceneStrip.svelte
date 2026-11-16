<script lang="ts">
	import { backend } from '$lib/backend';
	import type { SceneOutline } from '$lib/deck/outline';
	import { t } from '$lib/i18n';
	import SceneFrame from './SceneFrame.svelte';

	let {
		id,
		stamp,
		scenes,
		compiled,
		ratio,
		current,
		onpick
	}: {
		id: string;
		/** Names the stored state the thumbnails show: a new stamp loads them again. */
		stamp: string;
		scenes: SceneOutline[];
		/** How many scenes the stored revision has: a scene past it has no thumbnail yet. */
		compiled: number;
		ratio: number;
		current: number;
		onpick: (scene: SceneOutline) => void;
	} = $props();

	const ALL_STEPS = 100000;
</script>

<nav class="flex shrink-0 flex-col gap-1 rounded-fc-lg bg-fc-component p-2" aria-label={t('strip.title')}>
	<p class="px-1.5 text-fc-xs text-fc-fg-muted" data-testid="scene-count">
		{t('strip.count', { count: scenes.length })}
	</p>
	<ol class="flex gap-2 overflow-x-auto" data-testid="scene-strip">
		{#each scenes as scene (scene.index)}
			<li class="shrink-0">
				<button
					type="button"
					class="flex w-36 flex-col gap-1 rounded-fc-md p-1.5 text-left hover:bg-fc-surface focus-visible:outline-2
						focus-visible:outline-offset-2 focus-visible:outline-fc-ring sm:w-44
						{scene.index === current ? 'bg-fc-surface' : ''}"
					aria-current={scene.index === current ? 'true' : undefined}
					onclick={() => onpick(scene)}
				>
					<span
						class="block w-full overflow-hidden rounded-fc-sm bg-fc-surface shadow-sm"
						style="aspect-ratio: {ratio}"
					>
						{#if scene.index < compiled}
							<SceneFrame
								src={backend.frameUrl(id, { scene: scene.index, stamp })}
								title={scene.title || t('strip.scene', { scene: scene.index + 1 })}
								step={ALL_STEPS}
								lazy
							/>
						{/if}
					</span>
					<span class="flex items-baseline gap-1.5 px-0.5 text-fc-xs">
						<span class="font-semibold text-fc-fg tabular-nums">{scene.index + 1}</span>
						<span class="min-w-0 flex-1 truncate text-fc-fg-muted">{scene.layout}</span>
						{#if scene.steps > 0}
							<span class="whitespace-nowrap text-fc-fg-muted">{t('strip.steps', { count: scene.steps })}</span>
						{/if}
					</span>
				</button>
			</li>
		{/each}
	</ol>
</nav>
