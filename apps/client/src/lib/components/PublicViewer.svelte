<script lang="ts">
	import { Button, EmptyState, Icon, IconButton, Spinner, icons } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { Viewing } from '$lib/deck/viewing.svelte';
	import { t } from '$lib/i18n';
	import SceneFrame from './SceneFrame.svelte';
	import StageBox from './StageBox.svelte';

	let {
		token,
		embedded = false
	}: {
		token: string;
		/** Inside another site's frame: compact controls, and the only way out is the share page in a new tab. */
		embedded?: boolean;
	} = $props();

	const viewing = new Viewing();
	const playhead = viewing.playhead;

	$effect(() => void viewing.load(token));

	const position = $derived(playhead.position);
	const last = $derived(playhead.info?.steps ?? 0);

	function fullscreen() {
		if (document.fullscreenElement) void document.exitFullscreen();
		else void document.documentElement.requestFullscreen().catch(() => undefined);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.metaKey || event.ctrlKey || event.altKey) return;
		if (event.key.toLowerCase() === 'f') return fullscreen();
		if (viewing.key(event.key)) event.preventDefault();
	}
</script>

<svelte:head><title>{viewing.deck?.title ?? 'Diapo'} · Diapo</title></svelte:head>

<svelte:window {onkeydown} />

{#if viewing.gone}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page p-4" data-testid="link-gone">
		<EmptyState bare icon={icons.eyeClosed} title={t('public.gone')} description={t('public.gone.hint')} />
	</div>
{:else if viewing.failed}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page p-4">
		<EmptyState bare icon={icons.warning} title={t('error.generic')} description={t('error.hint')}>
			<Button variant="secondary" onclick={() => viewing.load(token)}>{t('action.retry')}</Button>
		</EmptyState>
	</div>
{:else}
	<div
		class="group relative h-dvh w-full overflow-hidden bg-fc-inverse"
		data-testid="viewer"
		data-step={playhead.at.step}
		data-scene={playhead.at.scene}
		data-last={last}
	>
		{#if !viewing.deck}
			<div class="flex h-full items-center justify-center"><Spinner /></div>
		{:else}
			<StageBox ratio={viewing.deck.width / viewing.deck.height}>
				<SceneFrame
					src={viewing.src}
					title={viewing.deck.title}
					step={playhead.want}
					onready={(info) => playhead.ready(info)}
					onstate={(state) => playhead.drawn(state)}
				/>
			</StageBox>
			<button
				type="button"
				class="absolute inset-y-0 left-0 w-1/3 focus-visible:outline-none"
				aria-label={t('step.previous')}
				data-testid="zone-previous"
				onclick={() => playhead.move(-1)}
			></button>
			<button
				type="button"
				class="absolute inset-y-0 right-0 w-2/3 focus-visible:outline-none"
				aria-label={t('step.next')}
				data-testid="zone-next"
				onclick={() => playhead.move(1)}
			></button>
			<div
				class="absolute inset-x-0 bottom-0 h-1 bg-fc-on-inverse/15"
				role="progressbar"
				aria-label={t('present.progress')}
				aria-valuemin="0"
				aria-valuemax={last}
				aria-valuenow={playhead.at.step}
			>
				<div
					class="h-full bg-fc-on-inverse/70 motion-safe:transition-[width]"
					style="width: {last > 0 ? (playhead.at.step / last) * 100 : 100}%"
				></div>
			</div>
			<div
				class="absolute flex items-center rounded-fc-pill bg-fc-bg shadow-lg {embedded
					? 'right-2 bottom-3 gap-0 p-0.5'
					: 'bottom-4 left-1/2 -translate-x-1/2 gap-1 p-1'}"
				data-testid="viewer-controls"
			>
				{#if !embedded}
					<IconButton variant="ghost" aria-label={t('step.previous')} onclick={() => playhead.move(-1)}>
						<Icon icon={icons.chevronLeft} size={18} />
					</IconButton>
				{/if}
				<span
					class="min-w-12 px-2 text-center text-fc-sm whitespace-nowrap text-fc-fg tabular-nums"
					data-testid="viewer-position"
				>
					{position.scene} / {position.scenes || viewing.deck.scenes}
				</span>
				{#if !embedded}
					<IconButton variant="ghost" aria-label={t('step.next')} onclick={() => playhead.move(1)}>
						<Icon icon={icons.chevronLeft} size={18} class="rotate-180" />
					</IconButton>
				{/if}
				{#if embedded}
					<button
						type="button"
						class="flex size-9 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface
							focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
						aria-label={t('present.fullscreen')}
						title={t('present.fullscreen')}
						onclick={fullscreen}
					>
						<Icon icon={icons.monitor} size={16} />
					</button>
					<a
						class="flex size-9 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface
							focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
						href="/p/{token}"
						target="_blank"
						rel="noopener"
						aria-label={t('public.open')}
						title={t('public.open')}
						data-testid="viewer-open"
					>
						<Icon icon={icons.external} size={16} />
					</a>
				{:else}
					<IconButton variant="ghost" aria-label={t('present.fullscreen')} onclick={fullscreen}>
						<Icon icon={icons.monitor} size={18} />
					</IconButton>
				{/if}
			</div>
			{#if !embedded}
				<a
					class="absolute top-3 right-3 flex min-h-11 items-center gap-2 rounded-fc-pill bg-fc-bg px-3 text-fc-xs text-fc-fg
						shadow-lg focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
					href="/"
					data-sveltekit-reload
					data-testid="made-with"
				>
					<Icon icon={logoIcon} size={16} />{t('public.made')}
				</a>
			{/if}
		{/if}
	</div>
{/if}
