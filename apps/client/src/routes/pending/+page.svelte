<script lang="ts">
	import { page } from '$app/state';
	import { Alert, ApprovalCard, Button, EmptyState, PageHeader, Spinner, StatusDot, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatMoment, t } from '$lib/i18n';
	import { STATES, needsBrowserLogin } from '$lib/pending';
	import { rememberReturn } from '$lib/return';
	import type { PendingAction } from '$lib/types';

	const id = $derived(page.url.searchParams.get('action') ?? '');

	let action = $state<PendingAction | null | undefined>(undefined);
	let outcome = $state<'confirmed' | 'dismissed' | null>(null);
	let busy = $state(false);
	let browserOnly = $state(false);
	let truncated = $state(false);

	function failed(error: unknown) {
		if (needsBrowserLogin(error)) browserOnly = true;
		else toast.danger(error instanceof Error && error.message ? error.message : t('error.generic'));
	}

	function signIn() {
		rememberReturn(page.url);
		window.location.assign(backend.loginUrl());
	}

	async function load(wanted: string) {
		action = undefined;
		outcome = null;
		try {
			const list = await backend.actions();
			truncated = list.truncated === true;
			action = list.actions.find((item) => item.id === wanted) ?? null;
		} catch (error) {
			action = null;
			failed(error);
		}
	}

	$effect(() => void load(id));

	const status = $derived(action ? (STATES[action.state] ?? STATES.missing) : null);

	async function answer(confirm: boolean) {
		if (!action) return;
		busy = true;
		try {
			await (confirm ? backend.confirmAction(action.id) : backend.dismissAction(action.id));
			outcome = confirm ? 'confirmed' : 'dismissed';
		} catch (error) {
			failed(error);
		}
		busy = false;
	}
</script>

<svelte:head><title>{t('pending.title')} · Diapo</title></svelte:head>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 sm:px-6 md:py-10">
	<PageHeader title={t('pending.title')} description={t('pending.subtitle')} />
	{#if browserOnly}
		<Alert tone="warning" title={t('pending.browser')}>
			{t('pending.browser.hint')}
			{#snippet action()}<Button size="sm" onclick={signIn}>{t('pending.signin')}</Button>{/snippet}
		</Alert>
	{:else if action === undefined}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if outcome}
		<EmptyState
			icon={outcome === 'confirmed' ? icons.check : icons.close}
			title={t(`pending.${outcome}`)}
			description={t(`pending.${outcome}.hint`)}
		>
			<Button href="/">{t('deck.back')}</Button>
		</EmptyState>
	{:else if !action || !status}
		<EmptyState
			icon={icons.warning}
			title={truncated ? t('pending.truncated') : t('pending.missing')}
			description={truncated ? t('pending.truncated.hint') : t('pending.missing.hint')}
		>
			<Button href="/">{t('deck.back')}</Button>
		</EmptyState>
	{:else}
		<div class="flex flex-wrap items-center gap-3 text-fc-xs text-fc-fg-muted">
			<StatusDot tone={status.tone} label={t(status.label)} />
			<span>{t('pending.client', { client: action.client_id })}</span>
			<span>{t('pending.expires', { moment: formatMoment(action.expires_at) })}</span>
			{#if action.state !== 'missing'}
				<a class="text-fc-fg underline" href="/decks/{action.target}">{t('pending.open')}</a>
			{/if}
		</div>
		<ApprovalCard
			approval={{ id: action.id, tool: action.kind, summary: action.summary, args: { deck_id: action.target } }}
			{busy}
			onApprove={() => answer(true)}
			onDecline={() => answer(false)}
		/>
	{/if}
</div>
