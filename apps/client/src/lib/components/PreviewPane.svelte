<script lang="ts">
	import { Icon, IconButton, icons } from '@facile/muse';
	import type { Playhead } from '$lib/deck/playhead.svelte';
	import { t } from '$lib/i18n';
	import SceneFrame from './SceneFrame.svelte';
	import StageBox from './StageBox.svelte';

	let {
		src,
		ratio,
		playhead
	}: {
		src: string;
		/** The stage's width over its height. */
		ratio: number;
		playhead: Playhead;
	} = $props();

	const position = $derived(playhead.position);
	const last = $derived(playhead.info?.steps ?? 0);
</script>

<section class="flex size-full min-h-0 flex-col gap-3 p-3" aria-label={t('preview.title')}>
	<div class="min-h-0 flex-1" data-testid="preview" data-step={playhead.at.step} data-scene={playhead.at.scene}>
		<StageBox {ratio} class="rounded-fc-md shadow-sm">
			<SceneFrame
				{src}
				title={t('preview.title')}
				step={playhead.want}
				onready={(info) => playhead.ready(info)}
				onstate={(state) => playhead.drawn(state)}
			/>
		</StageBox>
	</div>
	<div class="flex shrink-0 items-center justify-center gap-2">
		<IconButton
			variant="ghost"
			aria-label={t('step.previous')}
			disabled={playhead.want <= 0}
			onclick={() => playhead.move(-1)}
		>
			<Icon icon={icons.chevronLeft} size={18} />
		</IconButton>
		<span
			class="min-w-40 text-center text-fc-xs whitespace-nowrap text-fc-fg-muted tabular-nums"
			data-testid="position"
		>
			{t('position.scene', { scene: position.scene, scenes: position.scenes })}
			{#if position.steps > 0}· {t('position.step', { step: position.step, steps: position.steps })}{/if}
		</span>
		<IconButton
			variant="ghost"
			aria-label={t('step.next')}
			disabled={playhead.want >= last}
			onclick={() => playhead.move(1)}
		>
			<Icon icon={icons.chevronLeft} size={18} class="rotate-180" />
		</IconButton>
	</div>
</section>
