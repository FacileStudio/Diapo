<script lang="ts">
	import { Icon, Menu, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatMoment, t } from '$lib/i18n';
	import type { DeckSummary } from '$lib/types';
	import SceneFrame from './SceneFrame.svelte';

	let {
		deck,
		onrename,
		ontrash
	}: {
		deck: DeckSummary;
		onrename: () => void;
		ontrash: () => void;
	} = $props();

	const actions = $derived([
		{ label: t('deck.rename'), icon: icons.edit, onselect: onrename },
		{ label: t('deck.trash'), icon: icons.remove, tone: 'danger' as const, onselect: ontrash }
	]);
</script>

<article class="flex flex-col gap-1 rounded-fc-lg bg-fc-component p-2" data-testid="deck-card">
	<a
		href="/decks/{deck.id}"
		class="block aspect-video overflow-hidden rounded-fc-md focus-visible:outline-2 focus-visible:outline-offset-2
			focus-visible:outline-fc-ring"
		aria-label={deck.title}
	>
		<SceneFrame src={backend.frameUrl(deck.id, { scene: 0, stamp: deck.rev })} title={deck.title} lazy />
	</a>
	<div class="flex items-center gap-1 pl-2">
		<div class="min-w-0 flex-1 py-1">
			<h2 class="truncate text-fc-sm font-semibold text-fc-fg">{deck.title}</h2>
			<p class="truncate text-fc-xs text-fc-fg-muted">{formatMoment(deck.updated_at)}</p>
		</div>
		<Menu label={t('deck.actions', { title: deck.title })} items={actions} align="end">
			{#snippet trigger()}<Icon icon={icons.menu} size={18} />{/snippet}
		</Menu>
	</div>
</article>
