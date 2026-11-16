<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, Spinner, icons } from '@facile/muse';
	import { DeckLoader } from '$lib/deck/loader.svelte';
	import { t } from '$lib/i18n';
	import Workbench from '$lib/components/Workbench.svelte';

	const loader = new DeckLoader();

	$effect(() => void loader.load(page.params.id ?? ''));
</script>

{#if loader.missing}
	<div class="mx-auto flex w-full max-w-3xl flex-col px-4 py-16 sm:px-6">
		<EmptyState icon={icons.warning} title={t('deck.missing')} description={t('deck.missing.hint')}>
			<Button href="/">{t('deck.back')}</Button>
		</EmptyState>
	</div>
{:else if !loader.deck}
	<div class="flex h-full items-center justify-center"><Spinner /></div>
{:else}
	{#key loader.deck.id}
		<Workbench deck={loader.deck} />
	{/key}
{/if}
