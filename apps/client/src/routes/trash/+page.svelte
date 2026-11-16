<script lang="ts">
	import { Alert, Button, ConfirmModal, EmptyState, Spinner, icons, toast } from '@facile/muse';
	import { Shelf } from '$lib/deck/shelf.svelte';
	import { formatMoment, t } from '$lib/i18n';
	import type { DeckSummary } from '$lib/types';

	const shelf = new Shelf(true);

	let doomed = $state<DeckSummary | null>(null);

	void shelf.load();

	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	async function destroy() {
		const target = doomed;
		doomed = null;
		if (target) await shelf.destroy(target.id).catch(failed);
	}
</script>

<svelte:head><title>{t('trash.title')} · Diapo</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-lg flex-col gap-8 px-4 py-8 pb-24 sm:px-6 md:px-10 md:py-10 md:pb-32">
	<header class="flex flex-col gap-1">
		<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">{t('trash.title')}</h1>
		<p class="text-fc-sm text-fc-fg-muted">{t('trash.subtitle')}</p>
	</header>

	{#if !shelf.ready}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if shelf.failed}
		<Alert tone="danger">{t('decks.failed')}</Alert>
	{:else if shelf.decks.length === 0}
		<EmptyState icon={icons.remove} title={t('trash.empty')} />
	{:else}
		<ul class="flex flex-col gap-2" data-testid="trash-list">
			{#each shelf.decks as deck (deck.id)}
				<li class="flex flex-wrap items-center gap-2 rounded-fc-lg bg-fc-component py-2 pr-2 pl-4">
					<div class="min-w-0 flex-1">
						<h2 class="truncate text-fc-sm font-semibold text-fc-fg">{deck.title}</h2>
						<p class="truncate text-fc-xs text-fc-fg-muted">{formatMoment(deck.updated_at)}</p>
					</div>
					<Button size="sm" variant="secondary" onclick={() => shelf.restore(deck.id).catch(failed)}>
						{t('trash.restore')}
					</Button>
					<Button size="sm" variant="ghost-danger" onclick={() => (doomed = deck)}>{t('trash.delete')}</Button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<ConfirmModal
	open={doomed !== null}
	tone="danger"
	title={t('trash.confirm', { title: doomed?.title ?? '' })}
	description={t('trash.confirm.hint')}
	confirmLabel={t('trash.delete')}
	cancelLabel={t('action.cancel')}
	onConfirm={destroy}
	onCancel={() => (doomed = null)}
/>
