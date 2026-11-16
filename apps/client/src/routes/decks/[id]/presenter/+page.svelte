<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, Icon, IconButton, Spinner, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { DeckLoader } from '$lib/deck/loader.svelte';
	import { outline } from '$lib/deck/outline';
	import { Show, Stopwatch, clock } from '$lib/deck/show.svelte';
	import { revealed } from '$lib/deck/stage';
	import { t } from '$lib/i18n';
	import SceneFrame from '$lib/components/SceneFrame.svelte';
	import StageBox from '$lib/components/StageBox.svelte';

	const id = $derived(page.params.id ?? '');
	const loader = new DeckLoader();
	const show = new Show();
	const playhead = show.playhead;
	const watch = new Stopwatch();

	$effect(() => void loader.load(id));
	$effect(() => show.join(id));
	$effect(() => () => watch.pause());

	const plan = $derived(outline(loader.deck?.source ?? ''));
	const ratio = $derived(plan.width / plan.height);
	const src = $derived(backend.frameUrl(id, { stamp: loader.deck?.rev }));
	const position = $derived(playhead.position);
	const note = $derived(plan.scenes[playhead.at.scene]?.note ?? '');
	const upcoming = $derived(playhead.at.scene + 1 < position.scenes ? playhead.at.scene + 1 : null);

	function onkeydown(event: KeyboardEvent) {
		if (event.metaKey || event.ctrlKey || event.altKey) return;
		if (show.key(event.key)) event.preventDefault();
	}
</script>

<svelte:head><title>{t('presenter.title')} · {loader.deck?.title ?? 'Diapo'}</title></svelte:head>

<svelte:window {onkeydown} />

{#if loader.missing}
	<div class="mx-auto max-w-3xl px-4 py-16"><EmptyState icon={icons.warning} title={t('deck.missing')} /></div>
{:else if !loader.deck}
	<div class="flex h-dvh items-center justify-center bg-fc-page"><Spinner /></div>
{:else}
	<div
		class="flex min-h-dvh flex-col gap-3 bg-fc-page p-3 lg:h-dvh"
		data-testid="presenter"
		data-step={playhead.at.step}
	>
		<header class="flex flex-wrap items-center gap-2">
			<h1 class="min-w-0 flex-1 truncate text-fc-lg font-semibold text-fc-fg">{loader.deck.title}</h1>
			<span class="text-fc-sm whitespace-nowrap text-fc-fg-muted tabular-nums" data-testid="presenter-position">
				{t('position.scene', { scene: position.scene, scenes: position.scenes })}
				{#if position.steps > 0}· {t('position.step', { step: position.step, steps: position.steps })}{/if}
			</span>
			<span class="min-w-20 text-right text-fc-2xl font-semibold text-fc-fg tabular-nums" data-testid="timer">
				{clock(watch.elapsed)}
			</span>
			<Button size="sm" variant="secondary" onclick={() => watch.toggle()}>
				{watch.running ? t('timer.pause') : t('timer.start')}
			</Button>
			<IconButton variant="ghost" aria-label={t('timer.reset')} onclick={() => watch.reset()}>
				<Icon icon={icons.refresh} size={18} />
			</IconButton>
		</header>
		<div class="flex min-h-0 flex-1 flex-col gap-3 lg:grid lg:grid-cols-3 lg:grid-rows-[minmax(0,1fr)_minmax(0,1fr)]">
			<section class="aspect-video rounded-fc-lg bg-fc-component p-3 lg:col-span-2 lg:row-span-2 lg:aspect-auto">
				<StageBox {ratio} class="rounded-fc-md">
					<SceneFrame
						{src}
						title={t('presenter.current')}
						step={playhead.want}
						onready={(info) => playhead.ready(info)}
						onstate={(state) => playhead.drawn(state)}
					/>
				</StageBox>
			</section>
			<section class="flex flex-col gap-2 rounded-fc-lg bg-fc-component p-3">
				<h2 class="text-fc-xs font-semibold text-fc-fg-muted">{t('presenter.next')}</h2>
				<div class="aspect-video min-h-0 lg:aspect-auto lg:flex-1">
					{#if upcoming === null}
						<p class="flex size-full items-center justify-center text-fc-sm text-fc-fg-muted">{t('presenter.end')}</p>
					{:else}
						<StageBox {ratio} class="rounded-fc-md">
							<SceneFrame {src} title={t('presenter.next')} step={revealed(playhead.info, upcoming)} />
						</StageBox>
					{/if}
				</div>
			</section>
			<section class="flex min-h-40 flex-col gap-2 overflow-auto rounded-fc-lg bg-fc-component p-3">
				<h2 class="text-fc-xs font-semibold text-fc-fg-muted">{t('presenter.notes')}</h2>
				<p class="text-fc-lg whitespace-pre-wrap text-fc-fg" data-testid="notes">
					{#if note}{note}{:else}<span class="text-fc-sm text-fc-fg-muted">{t('presenter.nonotes')}</span>{/if}
				</p>
			</section>
		</div>
		<footer class="flex gap-3">
			<Button size="lg" variant="secondary" icon={icons.chevronLeft} class="flex-1" onclick={() => show.move(-1)}>
				{t('step.previous')}
			</Button>
			<Button size="lg" class="flex-1" onclick={() => show.move(1)}>{t('step.next')}</Button>
		</footer>
	</div>
{/if}
