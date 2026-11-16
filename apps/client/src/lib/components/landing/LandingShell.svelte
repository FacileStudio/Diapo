<script lang="ts">
	import type { Snippet } from 'svelte';
	import { LandingFooter, LandingHeader, SignInWithFacile, signUpUrl } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import { locale, rememberLocale, t } from '$lib/i18n';

	let { children }: { children: Snippet } = $props();

	const links = $derived([
		{ href: '/#features', label: t('landing.nav.features') },
		{ href: '/#pricing', label: t('landing.nav.pricing') }
	]);
	const columns = $derived([
		{ title: 'Diapo', links: [...links, { href: '/downloads', label: t('landing.nav.downloads') }] },
		{
			title: 'Facile',
			links: [
				{ href: 'https://suite.facile.studio', label: t('landing.nav.suite'), external: true },
				{ href: signUpUrl(locale()), label: t('landing.nav.signup'), external: true }
			]
		}
	]);
</script>

<div class="min-h-dvh overflow-x-clip bg-fc-page text-fc-fg" data-sveltekit-reload>
	<LandingHeader appName="Diapo" icon={logoIcon} {links} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={backend.loginUrl()} size="sm" />
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-col gap-20 pb-24 md:gap-28">{@render children()}</main>

	<LandingFooter
		appName="Diapo"
		icon={logoIcon}
		tagline={t('landing.line')}
		{columns}
		legal={{ onLocaleChange: rememberLocale }}
	/>
</div>
