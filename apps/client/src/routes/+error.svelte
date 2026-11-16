<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, icons } from '@facile/muse';
	import { t } from '$lib/i18n';

	const missing = $derived(page.status === 404);
</script>

<svelte:head><title>{missing ? t('error.missing') : t('error.title')} · Diapo</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col px-4 py-16 sm:px-6">
	<EmptyState
		icon={missing ? icons.search : icons.warning}
		title={missing ? t('error.missing') : t('error.generic')}
		description={missing ? t('error.missing.hint') : t('error.hint')}
	>
		<Button href="/" icon={icons.home}>{t('deck.back')}</Button>
	</EmptyState>
</div>
