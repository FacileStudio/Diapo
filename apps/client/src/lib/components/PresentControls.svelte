<script lang="ts">
	import { Icon, IconButton, icons } from '@facile/muse';
	import type { Show } from '$lib/deck/show.svelte';
	import { t } from '$lib/i18n';

	let {
		id,
		show,
		active,
		onfullscreen
	}: {
		id: string;
		show: Show;
		/** Whether the pointer moved lately: the bar hides while it rests. */
		active: boolean;
		onfullscreen: () => void;
	} = $props();

	const position = $derived(show.playhead.position);

	function presenter() {
		window.open(`/decks/${id}/presenter`, `diapo-presenter-${id}`, 'popup,width=1100,height=720');
	}
</script>

<div
	class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1 rounded-fc-pill bg-fc-bg p-1 shadow-lg
		focus-within:opacity-100 motion-safe:transition-opacity {active ? 'opacity-100' : 'pointer-events-none opacity-0'}"
	data-testid="present-controls"
>
	<IconButton variant="ghost" aria-label={t('step.previous')} onclick={() => show.move(-1)}>
		<Icon icon={icons.chevronLeft} size={18} />
	</IconButton>
	<span
		class="min-w-14 text-center text-fc-sm whitespace-nowrap text-fc-fg tabular-nums"
		data-testid="present-position"
	>
		{position.scene} / {position.scenes}
	</span>
	<IconButton variant="ghost" aria-label={t('step.next')} onclick={() => show.move(1)}>
		<Icon icon={icons.chevronLeft} size={18} class="rotate-180" />
	</IconButton>
	<IconButton variant="ghost" aria-label={t('presenter.open')} onclick={presenter}>
		<Icon icon={icons.eye} size={18} />
	</IconButton>
	<IconButton variant="ghost" aria-label={t('present.fullscreen')} onclick={onfullscreen}>
		<Icon icon={icons.monitor} size={18} />
	</IconButton>
	<a
		class="flex size-11 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface focus-visible:outline-2
			focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
		href="/decks/{id}"
		aria-label={t('present.exit')}
		title={t('present.exit')}
	>
		<Icon icon={icons.close} size={18} />
	</a>
</div>
