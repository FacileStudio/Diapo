<script lang="ts">
	import { goto } from '$app/navigation';
	import { Alert, Button, EmptyState, Spinner, icons, toast } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { Shelf } from '$lib/deck/shelf.svelte';
	import { t } from '$lib/i18n';
	import type { DeckSummary } from '$lib/types';
	import DeckCard from '$lib/components/DeckCard.svelte';
	import TitleDialog from '$lib/components/TitleDialog.svelte';

	const shelf = new Shelf();

	let dialog = $state<{ deck: DeckSummary | null } | null>(null);

	void shelf.load();

	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	async function create(title: string) {
		const deck = await shelf.create(title);
		await goto(`/decks/${deck.id}`);
	}

	async function submit(title: string) {
		const target = dialog?.deck;
		try {
			if (target) await shelf.rename(target.id, title);
			else await create(title);
			dialog = null;
		} catch (error) {
			failed(error);
		}
	}

	async function trash(deck: DeckSummary) {
		try {
			await shelf.trash(deck.id);
			const undo = { label: t('action.undo'), onClick: () => void shelf.restore(deck.id).catch(failed) };
			toast.success(t('deck.trashed', { title: deck.title }), { action: undo });
		} catch (error) {
			failed(error);
		}
	}
</script>

<svelte:head><title>{t('decks.title')} · Diapo</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 py-8 pb-24 sm:px-6 md:px-10 md:py-10 md:pb-32">
	<header class="flex flex-wrap items-center gap-4">
		<div class="flex min-w-0 flex-1 flex-col gap-1">
			<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">{t('decks.title')}</h1>
			<p class="text-fc-sm text-fc-fg-muted">{t('decks.subtitle')}</p>
		</div>
		<Button icon={icons.plus} onclick={() => (dialog = { deck: null })} data-testid="new-deck">{t('decks.new')}</Button>
	</header>

	{#if !shelf.ready}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if shelf.failed}
		<Alert tone="danger">
			{t('decks.failed')}
			{#snippet action()}
				<Button size="sm" variant="secondary" onclick={() => shelf.load()}>{t('action.retry')}</Button>
			{/snippet}
		</Alert>
	{:else if shelf.decks.length === 0}
		<EmptyState icon={logoIcon} title={t('decks.empty')} description={t('decks.empty.hint')}>
			<Button icon={icons.plus} onclick={() => (dialog = { deck: null })}>{t('decks.new')}</Button>
		</EmptyState>
	{:else}
		<ul class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" data-testid="deck-grid">
			{#each shelf.decks as deck (deck.id)}
				<li><DeckCard {deck} onrename={() => (dialog = { deck })} ontrash={() => trash(deck)} /></li>
			{/each}
		</ul>
	{/if}
</div>

<TitleDialog
	open={dialog !== null}
	heading={dialog?.deck ? t('deck.rename') : t('decks.new')}
	value={dialog?.deck?.title}
	confirm={dialog?.deck ? t('action.save') : t('action.create')}
	onsubmit={submit}
	onclose={() => (dialog = null)}
/>
