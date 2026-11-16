<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {
		AuthScreen,
		Icon,
		MobileNav,
		SideBar,
		Spinner,
		Toaster,
		Topbar,
		icons,
		provideLocale,
		type UserMenuOptions
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { logoIcon } from '$lib/brand';
	import { session } from '$lib/session.svelte';
	import { restoreTheme, saveTheme } from '$lib/theme';

	let { children } = $props();

	let collapsed = $state(typeof window !== 'undefined' && window.innerWidth < 1280);

	const path = $derived(page.url.pathname);

	provideLocale(() => 'fr');
	restoreTheme();

	$effect(() => {
		if (!session.ready) void session.load();
	});

	const pages = $derived([{ label: 'Présentations', href: '/', icon: icons.home, active: path === '/' }]);
	const navUser = $derived({
		name: session.user?.name || session.user?.email || 'Compte',
		email: session.user?.email,
		avatar: session.user?.avatar_url || undefined
	});
	const userMenu = $derived<UserMenuOptions>({
		showLanguage: false,
		onThemeChange: saveTheme,
		onSignOut: async () => {
			await session.logout();
			await goto('/');
		}
	});
</script>

{#if !session.ready}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page"><Spinner /></div>
{:else if !session.user}
	<AuthScreen appName="Diapo" href={backend.loginUrl()} />
{:else}
	<div class="flex h-dvh w-full overflow-hidden bg-fc-page">
		<div class="hidden h-full shrink-0 p-3 md:flex">
			<SideBar icon={logoIcon} title="Diapo" bind:collapsed {pages} user={navUser} {userMenu} class="h-full" />
		</div>

		<main class="min-w-0 flex-1 overflow-auto overscroll-contain pb-28 md:pb-0">
			<Topbar class="md:hidden">
				<span class="flex flex-1 items-center gap-2 text-fc-md font-semibold text-fc-fg">
					<Icon icon={logoIcon} size={20} />Diapo
				</span>
			</Topbar>
			{@render children()}
		</main>

		<MobileNav items={pages} user={navUser} {userMenu} />
	</div>
{/if}

<Toaster />
