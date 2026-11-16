<script lang="ts">
	import { untrack } from 'svelte';
	import { Alert, Button, ConfirmModal, CopyButton, Field, Input, Modal, Spinner, toast } from '@facile/muse';
	import { Sharing, embedSnippet } from '$lib/deck/sharing.svelte';
	import { formatMoment, t } from '$lib/i18n';

	let {
		open = $bindable(),
		id,
		title,
		width,
		height
	}: {
		open: boolean;
		id: string;
		title: string;
		/** The stage size, for the embed snippet's ratio. */
		width: number;
		height: number;
	} = $props();

	const sharing = new Sharing(untrack(() => id));

	let rotating = $state(false);
	let revoking = $state(false);

	const embed = $derived(sharing.url ? embedSnippet(sharing.url, title, width, height) : '');
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	$effect(() => {
		if (open) void sharing.load();
		else sharing.forget();
	});

	async function rotate() {
		rotating = false;
		await sharing.create(true).catch(failed);
	}
</script>

<Modal {open} title={t('share.title')} size="md" onClose={() => (open = false)}>
	<div class="flex flex-col gap-4" data-testid="share-dialog" data-status={sharing.status}>
		{#if sharing.status === 'loading'}
			<div class="flex justify-center py-6"><Spinner /></div>
		{:else if sharing.status === 'failed'}
			<Alert tone="danger">{t('share.failed')}</Alert>
		{:else if sharing.status === 'off'}
			<p class="text-fc-sm text-fc-fg-muted">{t('share.off')}</p>
			<Button disabled={sharing.busy} onclick={() => sharing.create().catch(failed)} data-testid="share-create">
				{t('share.create')}
			</Button>
		{:else}
			{#if sharing.url}
				<Alert tone="warning" title={t('share.once')}>{t('share.once.hint')}</Alert>
				<Field label={t('share.link')}>
					{#snippet children({ id: field })}
						<span class="flex items-center gap-2">
							<Input
								id={field}
								value={sharing.url ?? ''}
								readonly
								class="font-fc-mono text-fc-sm"
								data-testid="share-url"
							/>
							<CopyButton text={sharing.url ?? ''} label={t('share.copy')} variant="secondary" />
						</span>
					{/snippet}
				</Field>
				<Field label={t('share.embed')} helper={t('share.embed.hint')}>
					{#snippet children({ id: field })}
						<span class="flex items-center gap-2">
							<Input id={field} value={embed} readonly class="font-fc-mono text-fc-sm" data-testid="share-embed" />
							<CopyButton text={embed} label={t('share.embed.copy')} variant="secondary" />
						</span>
					{/snippet}
				</Field>
			{:else}
				<p class="text-fc-sm text-fc-fg" data-testid="share-live">
					{t('share.on', { moment: formatMoment(sharing.createdAt ?? '') })}
				</p>
				<p class="text-fc-sm text-fc-fg-muted">{t('share.hidden')}</p>
			{/if}
			<div class="flex flex-wrap gap-2">
				<Button
					variant="secondary"
					disabled={sharing.busy}
					onclick={() => (rotating = true)}
					data-testid="share-rotate"
				>
					{t('share.rotate')}
				</Button>
				<Button
					variant="ghost-danger"
					disabled={sharing.busy}
					onclick={() => (revoking = true)}
					data-testid="share-revoke"
				>
					{t('share.revoke')}
				</Button>
			</div>
		{/if}
	</div>
</Modal>

<ConfirmModal
	open={rotating}
	tone="danger"
	title={t('share.rotate.title')}
	description={t('share.rotate.hint')}
	confirmLabel={t('share.rotate')}
	cancelLabel={t('action.cancel')}
	onConfirm={rotate}
	onCancel={() => (rotating = false)}
/>

<ConfirmModal
	open={revoking}
	tone="danger"
	title={t('share.revoke.title')}
	description={t('share.revoke.hint')}
	confirmLabel={t('share.revoke')}
	cancelLabel={t('action.cancel')}
	onConfirm={() => {
		revoking = false;
		sharing.revoke().catch(failed);
	}}
	onCancel={() => (revoking = false)}
/>
