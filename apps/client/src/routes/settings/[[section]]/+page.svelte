<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		PreferencesSection,
		ProfileSection,
		SettingsLayout,
		icons,
		readTheme,
		sectionHref,
		settingsRoute
	} from '@facile/muse';
	import { t } from '$lib/i18n';
	import { session } from '$lib/session.svelte';
	import { saveTheme } from '$lib/theme';

	const sections = $derived([
		{ id: 'profile', label: t('settings.profile'), icon: icons.userCircle, href: sectionHref('/settings', 'profile') },
		{
			id: 'preferences',
			label: t('settings.preferences'),
			icon: icons.palette,
			href: sectionHref('/settings', 'preferences')
		}
	]);

	const route = $derived(settingsRoute(page.params.section, sections));

	$effect(() => {
		if (!route.known) void goto('/settings', { replaceState: true });
	});
</script>

<svelte:head><title>{t('settings.title')} · Diapo</title></svelte:head>

<div class="mx-auto w-full max-w-5xl px-4 py-6 sm:px-6 md:py-10">
	<SettingsLayout {sections} active={page.params.section} backHref="/settings" title={t('settings.title')}>
		{#snippet children(current)}
			{#if current === 'profile'}
				<ProfileSection
					title={t('settings.profile')}
					name={session.user?.name || session.user?.email || ''}
					email={session.user?.email}
					avatar={session.user?.avatar_url}
				/>
			{:else}
				<PreferencesSection title={t('settings.preferences')} theme={readTheme()} onThemeChange={saveTheme} />
			{/if}
		{/snippet}
	</SettingsLayout>
</div>
