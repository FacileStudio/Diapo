<script lang="ts">
	import { Icon, IconButton, Menu, icons, type MenuEntry } from '@facile/muse';
	import { EditorActions } from '$lib/editor/actions';
	import type { DeckEditor, Insertable } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import type { Asset } from '$lib/types';
	import { spell } from './menus';
	import { tools } from './tools';

	let {
		editor,
		assets,
		onfiles,
		onhelp
	}: {
		editor: DeckEditor;
		/** The deck's files: the image and video menus list them. */
		assets: Asset[];
		/** Opens the files panel, to upload one. */
		onfiles: () => void;
		/** Opens the list of keyboard shortcuts. */
		onhelp: () => void;
	} = $props();

	const SHAPES = [
		{ kind: 'text', icon: icons.text, label: 'tool.text' },
		{ kind: 'rect', icon: tools.rect, label: 'tool.rect' },
		{ kind: 'ellipse', icon: tools.ellipse, label: 'tool.ellipse' },
		{ kind: 'diamond', icon: tools.diamond, label: 'tool.diamond' }
	] as const;

	const ready = $derived(editor.synced && !editor.stopped && editor.current !== null);
	const picked = $derived(editor.selection.length > 0);
	const actions = $derived(new EditorActions(editor));
	const hint = (label: string, keys: string) => `${label} (${spell(keys)})`;

	const insert = (kind: Insertable, asset?: string) => void editor.insert(kind, undefined, asset);

	function media(kind: 'image' | 'video'): MenuEntry[] {
		const found = assets.filter((asset) => asset.mime.startsWith(`${kind}/`));
		const files = found.map((asset) => ({ label: asset.name, onselect: () => insert(kind, asset.name) }));
		return [...files, ...(files.length ? [{ separator: true as const }] : []), { label: t('tool.files'), onselect: onfiles }];
	}

	const button = 'flex items-center gap-1.5';
</script>

<div
	class="flex shrink-0 flex-wrap items-center gap-1 rounded-fc-lg bg-fc-component p-1"
	role="toolbar"
	aria-label={t('tool.label')}
	data-testid="toolbar"
>
	{#each SHAPES as shape (shape.kind)}
		<IconButton
			variant="ghost"
			aria-label={t(shape.label)}
			tooltip={t('tool.insertHint', { what: t(shape.label) })}
			disabled={!ready}
			data-testid="tool-{shape.kind}"
			onclick={() => insert(shape.kind)}
		>
			<Icon icon={shape.icon} size={18} />
		</IconButton>
	{/each}
	{#each ['image', 'video'] as const as kind (kind)}
		<Menu label={t(`tool.${kind}`)} items={media(kind)} data-testid="tool-{kind}">
			{#snippet trigger()}
				<span class={button}><Icon icon={tools[kind]} size={18} /><span class="max-lg:sr-only">{t(`tool.${kind}`)}</span></span>
			{/snippet}
		</Menu>
	{/each}
	<span class="mx-1 h-6 w-0.5 rounded-fc-pill bg-fc-surface" aria-hidden="true"></span>
	<IconButton
		variant="ghost"
		aria-label={t('tool.front')}
		tooltip={hint(t('tool.front'), 'mod+shift+]')}
		disabled={!picked}
		onclick={() => actions.reorder('front')}
	>
		<Icon icon={tools.front} size={18} />
	</IconButton>
	<IconButton
		variant="ghost"
		aria-label={t('tool.back')}
		tooltip={hint(t('tool.back'), 'mod+shift+[')}
		disabled={!picked}
		onclick={() => actions.reorder('back')}
	>
		<Icon icon={tools.back} size={18} />
	</IconButton>
	<IconButton
		variant="ghost"
		aria-label={t('tool.undo')}
		tooltip={hint(t('tool.undo'), 'mod+z')}
		disabled={!editor.canUndo}
		onclick={() => editor.undo()}
	>
		<Icon icon={tools.undo} size={18} />
	</IconButton>
	<IconButton
		variant="ghost"
		aria-label={t('tool.redo')}
		tooltip={hint(t('tool.redo'), 'mod+shift+z')}
		disabled={!editor.canRedo}
		onclick={() => editor.redo()}
	>
		<Icon icon={tools.redo} size={18} />
	</IconButton>
	<IconButton
		variant="ghost-danger"
		aria-label={t('tool.delete')}
		tooltip={hint(t('tool.delete'), 'delete')}
		disabled={!picked}
		data-testid="tool-delete"
		onclick={() => editor.remove(editor.selection)}
	>
		<Icon icon={icons.remove} size={18} />
	</IconButton>
	<span class="flex-1" aria-hidden="true"></span>
	<IconButton
		variant="ghost"
		aria-label={t('menu.shortcuts')}
		tooltip={hint(t('menu.shortcuts'), '?')}
		data-testid="shortcuts-open"
		onclick={onhelp}
	>
		<Icon icon={icons.help} size={18} />
	</IconButton>
</div>
