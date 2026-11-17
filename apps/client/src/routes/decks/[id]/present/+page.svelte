<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Spinner } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { DeckLoader } from '$lib/deck/loader.svelte';
	import { readPlan } from '$lib/deck/projection';
	import { Show } from '$lib/deck/show.svelte';
	import { t } from '$lib/i18n';
	import PresentControls from '$lib/components/PresentControls.svelte';
	import SceneFrame from '$lib/components/SceneFrame.svelte';
	import StageBox from '$lib/components/StageBox.svelte';

	const id = $derived(page.params.id ?? '');
	const loader = new DeckLoader();
	const show = new Show();
	const playhead = show.playhead;

	let active = $state(true);
	let idle: ReturnType<typeof setTimeout> | undefined;

	$effect(() => void loader.load(id));
	$effect(() => show.join(id));
	$effect(() => {
		if (loader.missing) void goto(`/decks/${id}`, { replaceState: true });
	});

	const plan = $derived(readPlan(loader.deck?.source ?? ''));
	const last = $derived(playhead.info?.steps ?? 0);

	function wake() {
		active = true;
		clearTimeout(idle);
		idle = setTimeout(() => (active = false), 2500);
	}

	function fullscreen() {
		if (document.fullscreenElement) void document.exitFullscreen();
		else void document.documentElement.requestFullscreen().catch(() => undefined);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.metaKey || event.ctrlKey || event.altKey) return;
		if (event.key === 'Escape' && !document.fullscreenElement) return void goto(`/decks/${id}`);
		if (event.key.toLowerCase() === 'f') return fullscreen();
		if (show.key(event.key)) event.preventDefault();
	}
</script>

<svelte:head><title>{loader.deck?.title ?? t('present.start')} · Diapo</title></svelte:head>

<svelte:window {onkeydown} onpointermove={wake} />

<div
	class="relative h-dvh w-full overflow-hidden bg-fc-inverse {active ? '' : 'cursor-none'}"
	data-testid="present"
	data-step={playhead.at.step}
	data-scene={playhead.at.scene}
	data-last={last}
>
	{#if !loader.deck}
		<div class="flex h-full items-center justify-center"><Spinner /></div>
	{:else}
		<StageBox ratio={plan.width / plan.height}>
			<SceneFrame
				src={backend.frameUrl(id, { stamp: loader.deck.rev })}
				title={loader.deck.title}
				step={playhead.want}
				onready={(info) => playhead.ready(info)}
				onstate={(state) => playhead.drawn(state)}
			/>
		</StageBox>
		<button
			type="button"
			class="absolute inset-y-0 left-0 w-1/3 cursor-[inherit] focus-visible:outline-none"
			aria-label={t('step.previous')}
			data-testid="zone-previous"
			onclick={() => show.move(-1)}
		></button>
		<button
			type="button"
			class="absolute inset-y-0 right-0 w-2/3 cursor-[inherit] focus-visible:outline-none"
			aria-label={t('step.next')}
			data-testid="zone-next"
			onclick={() => show.move(1)}
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
		<PresentControls {id} {show} {active} onfullscreen={fullscreen} />
	{/if}
</div>
