<script lang="ts">
	import { CopyButton, Drawer, Dropzone, Icon, IconButton, Spinner, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { t } from '$lib/i18n';
	import type { Asset } from '$lib/types';

	let {
		open = $bindable(),
		id,
		assets = $bindable()
	}: {
		open: boolean;
		id: string;
		/** The deck's files, shared with the insert menu. */
		assets: Asset[];
	} = $props();

	let busy = $state(false);

	const reference = (asset: Asset) => `![](${asset.name})`;
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('assets.failed'));

	async function refresh() {
		assets = await backend.assets(id);
	}

	async function add(files: File[]) {
		busy = true;
		try {
			for (const file of files) await backend.upload(id, file);
			await refresh();
		} catch (error) {
			failed(error);
		}
		busy = false;
	}

	async function remove(asset: Asset) {
		try {
			await backend.removeAsset(id, asset.name);
			assets = assets.filter((known) => known.name !== asset.name);
		} catch (error) {
			failed(error);
		}
	}

	$effect(() => {
		if (open) void refresh().catch(failed);
	});
</script>

<Drawer {open} title={t('assets.title')} description={t('assets.hint')} onClose={() => (open = false)}>
	<div class="flex flex-col gap-4" data-testid="assets">
		<Dropzone multiple label={t('assets.drop')} browseLabel={t('assets.browse')} disabled={busy} onFiles={add} />
		{#if busy}<div class="flex justify-center"><Spinner /></div>{/if}
		{#if assets.length === 0 && !busy}
			<p class="text-fc-sm text-fc-fg-muted">{t('assets.empty')}</p>
		{/if}
		<ul class="flex flex-col gap-2">
			{#each assets as asset (asset.name)}
				<li class="flex items-center gap-2 rounded-fc-md bg-fc-surface py-1 pr-1 pl-3">
					<Icon icon={icons.folder} size={16} class="shrink-0 text-fc-fg-muted" />
					<span class="min-w-0 flex-1 truncate font-fc-mono text-fc-xs text-fc-fg">{asset.name}</span>
					<CopyButton text={reference(asset)} label={t('assets.copy')} />
					<IconButton
						variant="ghost-danger"
						aria-label={t('assets.delete', { name: asset.name })}
						onclick={() => remove(asset)}
					>
						<Icon icon={icons.remove} size={18} />
					</IconButton>
				</li>
			{/each}
		</ul>
	</div>
</Drawer>
