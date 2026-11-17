<script lang="ts">
	import { ContextMenu, Drawer, Dropzone, Icon, IconButton, Spinner, icons, toast, type MenuEntry } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { t } from '$lib/i18n';
	import type { Asset } from '$lib/types';

	let {
		open = $bindable(),
		id,
		assets = $bindable(),
		oninsert
	}: {
		open: boolean;
		id: string;
		/** The deck's files, shared with the toolbar. */
		assets: Asset[];
		/** Places a picture or a video on the current slide. */
		oninsert: (asset: Asset) => void;
	} = $props();

	let busy = $state(false);

	const placeable = (asset: Asset) => /^(image|video)\//.test(asset.mime);
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

	function entries(target: Element): MenuEntry[] {
		const name = target.closest<HTMLElement>('[data-asset]')?.dataset.asset;
		const asset = assets.find((known) => known.name === name);
		if (!asset) return [];
		const insert: MenuEntry[] = placeable(asset)
			? [{ label: t('assets.place'), icon: icons.plus, onselect: () => oninsert(asset) }, { separator: true }]
			: [];
		return [...insert, { label: t('assets.remove'), icon: icons.remove, tone: 'danger', onselect: () => void remove(asset) }];
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
		<ContextMenu items={entries} label={t('assets.menu')}>
		<ul class="flex flex-col gap-2">
			{#each assets as asset (asset.name)}
				<li class="flex items-center gap-2 rounded-fc-md bg-fc-surface py-1 pr-1 pl-3" data-asset={asset.name}>
					<Icon icon={icons.folder} size={16} class="shrink-0 text-fc-fg-muted" />
					<span class="min-w-0 flex-1 truncate font-fc-mono text-fc-xs text-fc-fg">{asset.name}</span>
					{#if placeable(asset)}
						<IconButton variant="ghost" aria-label={t('assets.insert', { name: asset.name })} onclick={() => oninsert(asset)}>
							<Icon icon={icons.plus} size={18} />
						</IconButton>
					{/if}
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
		</ContextMenu>
	</div>
</Drawer>
