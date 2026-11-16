<script lang="ts">
	import {
		FeatureSplit,
		ForDevelopers,
		LANDING_HEADING,
		LandingCta,
		PricingPlans,
		WorksWith,
		icons,
		signUpUrl,
		type DeveloperRow,
		type FeatureItem,
		type WorksWithApp
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { bulleIcon } from '$lib/brand';
	import { CLI_RELEASED, REPO_HREF } from '$lib/downloads';
	import { locale, t } from '$lib/i18n';
	import BulleTile from './BulleTile.svelte';
	import EditorDemo from './EditorDemo.svelte';
	import FileTile from './FileTile.svelte';
	import LandingShell from './LandingShell.svelte';
	import PresentTile from './PresentTile.svelte';
	import TerminalTile from './TerminalTile.svelte';

	const signInHref = backend.loginUrl();
	const signUpHref = $derived(signUpUrl(locale()));
	const FEATURES = [
		['steps', icons.layers],
		['notes', icons.eyeClosed],
		['save', icons.check],
		['layouts', icons.dashboard],
		['images', icons.upload],
		['exports', icons.download]
	] as const;

	const features = $derived<FeatureItem[]>(
		FEATURES.map(([key, icon]) => ({
			icon,
			title: t(`landing.feature.${key}.title`),
			text: t(`landing.feature.${key}.text`)
		}))
	);

	const apps = $derived<WorksWithApp[]>([
		{ name: 'Bulle', icon: bulleIcon, line: t('landing.with.bulle'), href: 'https://bulle.facile.studio' }
	]);

	const rows = $derived<DeveloperRow[]>([
		{ kind: 'mcp', text: t('landing.dev.mcp') },
		{ kind: 'api', text: t('landing.dev.api'), href: '/docs' },
		{ kind: 'cli', text: t(CLI_RELEASED ? 'landing.dev.cli' : 'landing.dev.cliSoon'), href: '/downloads' },
		{ kind: 'source', text: t('landing.dev.source'), href: REPO_HREF }
	]);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});

	const width = 'mx-auto w-full max-w-5xl px-4 sm:px-6';
</script>

<svelte:head><title>{t('landing.title')}</title></svelte:head>

<LandingShell>
	<section class="{width} flex flex-col items-center pt-16 text-center sm:pt-24">
		<h1 class="max-w-3xl font-fc-title text-fc-3xl leading-tight font-bold tracking-tight text-balance sm:text-5xl">
			{#each t('landing.slogan').split(/(?<=\.)\s/) as sentence (sentence)}<span class="block">{sentence}</span>{/each}
		</h1>
		<p class="mt-5 max-w-2xl text-fc-lg text-balance text-fc-fg-muted">{t('landing.subtitle')}</p>
		<LandingCta {signInHref} {signUpHref} align="center" class="mt-8 w-full" />
		<div class="mt-12 w-full sm:mt-16"><EditorDemo /></div>
	</section>

	<section aria-labelledby="tiles-title" class={width}>
		<h2 id="tiles-title" class="mb-10 text-center {LANDING_HEADING}">{t('landing.tiles')}</h2>
		<div class="grid gap-6 sm:grid-cols-2">
			<FileTile />
			<BulleTile />
			<PresentTile />
			<TerminalTile />
		</div>
	</section>

	<FeatureSplit
		id="features"
		class="{width} scroll-mt-24"
		title={t('landing.features')}
		description={t('landing.featuresLine')}
		{features}
	/>

	<WorksWith class={width} {apps} description={t('landing.with.line')} />

	<section class="{width} flex flex-col items-center gap-5 text-center">
		<h2 class={LANDING_HEADING}>{t('landing.cta.title')}</h2>
		<p class="text-fc-md text-fc-fg-muted">{t('landing.cta.line')}</p>
		<LandingCta {signInHref} {signUpHref} align="center" class="w-full" />
	</section>

	<PricingPlans
		id="pricing"
		class="{width} scroll-mt-24"
		title={t('landing.nav.pricing')}
		headingClass={LANDING_HEADING}
		highlight={['ai.access']}
		highlightOnly
		notes={{ '*': t('landing.plan.paid'), solo: t('landing.plan.included') }}
		recommended="pro"
		suiteHref="https://suite.facile.studio/#tarifs"
	/>

	<div class={width}>
		<ForDevelopers
			{rows}
			description={t('landing.dev.line')}
			selfHostHref={`https://suite.facile.studio/${locale() === 'fr' ? '' : `${locale()}/`}auto-hebergement`}
		/>
	</div>
</LandingShell>
