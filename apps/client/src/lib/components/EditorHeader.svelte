<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Button, ConnectionHint, Icon, IconButton, icons } from '@facile/muse';
	import { shareIcon } from '$lib/brand';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import PresenceBar from './editor/PresenceBar.svelte';

	let {
		id,
		title,
		editor,
		previewing,
		assistant,
		bulleOpen,
		onrename,
		onpreview,
		onassets,
		onshare,
		onexport,
		ontogglebulle
	}: {
		id: string;
		title: string;
		editor: DeckEditor;
		/** Whether the compiled preview replaces the stage. */
		previewing: boolean;
		/** The name of the person's Bulle assistant. */
		assistant: string;
		bulleOpen: boolean;
		onrename: (title: string) => void;
		onpreview: () => void;
		onassets: () => void;
		onshare: () => void;
		onexport: () => void;
		ontogglebulle: () => void;
	} = $props();

	let draft = $state('');

	$effect(() => {
		draft = title;
	});

	function commit() {
		const next = draft.trim();
		if (next && next !== title) onrename(next);
		else draft = title;
	}

	const ask = $derived(t('bulle.ask', { name: assistant }));
</script>

<header class="flex flex-wrap items-center gap-x-2 gap-y-1">
	<Button variant="ghost" size="sm" class="h-11" icon={icons.chevronLeft} href="/" aria-label={t('decks.title')}>
		<span class="hidden sm:inline">{t('decks.title')}</span>
	</Button>
	<input
		class="h-11 min-w-0 flex-1 rounded-fc-md bg-transparent px-2 text-fc-lg font-semibold text-fc-fg
			placeholder:text-fc-fg-muted hover:bg-fc-surface focus-visible:bg-fc-surface focus-visible:outline-2
			focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
		aria-label={t('deck.title')}
		placeholder={t('deck.untitled')}
		maxlength="200"
		bind:value={draft}
		onblur={commit}
		onkeydown={(event) => {
			if (event.key === 'Enter') event.currentTarget.blur();
		}}
	/>
	<ConnectionHint state={editor.connection} data-testid="connection" data-state={editor.connection} />
	<div class="ml-auto flex shrink-0 items-center gap-1">
		<PresenceBar {editor} />
		<Button
			variant="ghost"
			size="md"
			class="h-11 {previewing ? 'bg-fc-surface' : ''}"
			icon={icons.eye}
			aria-label={t('preview.title')}
			aria-pressed={previewing}
			data-testid="preview-toggle"
			onclick={onpreview}
		>
			<span class="hidden md:inline">{t('preview.title')}</span>
		</Button>
		<IconButton variant="ghost" aria-label={t('assets.title')} onclick={onassets}>
			<Icon icon={icons.folder} size={18} />
		</IconButton>
		<IconButton variant="ghost" aria-label={t('export.title')} onclick={onexport} data-testid="export-open">
			<Icon icon={icons.download} size={18} />
		</IconButton>
		<IconButton variant="ghost" aria-label={t('share.title')} onclick={onshare} data-testid="share-open">
			<Icon icon={shareIcon} size={18} />
		</IconButton>
		<button
			type="button"
			class="flex min-h-11 items-center gap-2 rounded-fc-md px-2 text-fc-sm text-fc-fg hover:bg-fc-surface
				focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring
				{bulleOpen ? 'bg-fc-surface' : ''}"
			aria-pressed={bulleOpen}
			aria-label={ask}
			data-testid="bulle-toggle"
			onclick={ontogglebulle}
		>
			<Blob name={assistant} size={24} />
			<span class="hidden xl:inline">{ask}</span>
		</button>
		<Button size="md" class="h-11" icon={icons.monitor} href="/decks/{id}/present" data-testid="present">
			<span class="hidden sm:inline">{t('present.start')}</span>
		</Button>
	</div>
</header>
