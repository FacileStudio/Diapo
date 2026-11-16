<script lang="ts">
	import { Button, EmptyState, icons } from '@facile/muse';
	import { CLI_RELEASED, REPO_HREF } from '$lib/downloads';
	import { t } from '$lib/i18n';
	import LandingShell from '$lib/components/landing/LandingShell.svelte';
</script>

<svelte:head><title>{t('downloads.title')}</title></svelte:head>

<LandingShell>
	<section class="mx-auto w-full max-w-4xl px-4 pt-12 sm:px-6 sm:pt-16" aria-labelledby="downloads-title">
		<div class="mb-8 text-center">
			<h1 id="downloads-title" class="font-fc-title text-fc-2xl font-bold tracking-tight sm:text-fc-3xl">
				{t('downloads.heading')}
			</h1>
			<p class="mt-2 text-fc-sm text-fc-fg-muted sm:text-fc-md">{t('downloads.body')}</p>
		</div>
		{#if !CLI_RELEASED}
			<EmptyState icon={icons.clock} title={t('downloads.soon')} description={t('downloads.soon.hint')}>
				<Button href={REPO_HREF} target="_blank" rel="noopener" variant="secondary" icon={icons.github}>
					{t('downloads.source')}
				</Button>
			</EmptyState>
		{/if}
	</section>
</LandingShell>
