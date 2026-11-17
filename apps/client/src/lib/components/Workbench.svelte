<script lang="ts">
	import { onDestroy, untrack } from 'svelte';
	import { page } from '$app/state';
	import { Spinner, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { assistant as askAssistant, DEFAULT_ASSISTANT, rememberSidebar, sidebarOpen } from '$lib/bulle';
	import { watchDeck } from '$lib/deck/events';
	import { Playhead } from '$lib/deck/playhead.svelte';
	import { createDeckEditor } from '$lib/editor/deck.svelte';
	import { t } from '$lib/i18n';
	import { session } from '$lib/session.svelte';
	import type { Asset, Deck } from '$lib/types';
	import AssetsPanel from './AssetsPanel.svelte';
	import BulleSidebar from './BulleSidebar.svelte';
	import ConnectionBanner from './editor/ConnectionBanner.svelte';
	import Inspector from './editor/Inspector.svelte';
	import NotesPanel from './editor/NotesPanel.svelte';
	import SlideRail from './editor/SlideRail.svelte';
	import Stage from './editor/Stage.svelte';
	import Toolbar from './editor/Toolbar.svelte';
	import EditorHeader from './EditorHeader.svelte';
	import ExportDialog from './ExportDialog.svelte';
	import PreviewPane from './PreviewPane.svelte';
	import ShareDialog from './ShareDialog.svelte';

	let { deck }: { deck: Deck } = $props();

	const id = untrack(() => deck.id);
	const owner = session.user?.email ?? '';
	const user = { name: session.user?.name || owner };
	const editor = createDeckEditor(id, user);
	const playhead = new Playhead();

	let title = $state(untrack(() => deck.title));
	let rev = $state(untrack(() => deck.rev));
	let touches = $state(0);
	let assets = $state<Asset[]>([]);
	let assetsOpen = $state(false);
	let shareOpen = $state(false);
	let exportOpen = $state(false);
	let previewing = $state(false);
	let notesOpen = $state(true);
	let assistant = $state(DEFAULT_ASSISTANT.name);
	let bulleOpen = $state(sidebarOpen(owner));

	const ratio = $derived(editor.size.width / editor.size.height);
	const stamp = $derived(`${rev}.${touches}`);
	const assetUrl = (name: string) => assets.find((asset) => asset.name === name)?.url ?? '';
	const loadAssets = () => backend.assets(id).then((found) => (assets = found));
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	$effect(() => {
		const index = Math.max(0, editor.slides.findIndex((slide) => slide.id === editor.current));
		untrack(() => playhead.focus(index));
	});

	$effect(() => rememberSidebar(owner, bulleOpen));

	$effect(() => watchDeck(id, changed));

	void askAssistant().then((found) => (assistant = found.name));
	void loadAssets().catch(() => undefined);

	onDestroy(() => editor.destroy());

	async function refresh() {
		try {
			await loadAssets();
			title = (await backend.deck(id)).title;
		} catch {
			return;
		}
	}

	function changed(next: number, first: boolean) {
		if (first) return;
		rev = next;
		touches += 1;
		void refresh();
	}

	async function rename(next: string) {
		try {
			title = (await backend.rename(id, next)).title;
		} catch (error) {
			failed(error);
		}
	}

	function place(asset: Asset) {
		const kind = asset.mime.startsWith('video/') ? 'video' : 'image';
		editor.insert(kind, undefined, asset.name);
		assetsOpen = false;
		previewing = false;
	}
</script>

<svelte:head><title>{title} · Diapo</title></svelte:head>

<div
	class="flex h-full min-h-0 flex-col gap-3 p-3 {bulleOpen ? 'lg:pr-[412px]' : ''}"
	data-testid="editor"
	data-connection={editor.connection}
	data-synced={editor.synced}
>
	<EditorHeader
		{id}
		{title}
		{editor}
		{previewing}
		{assistant}
		{bulleOpen}
		onrename={rename}
		onpreview={() => (previewing = !previewing)}
		onassets={() => (assetsOpen = true)}
		onshare={() => (shareOpen = true)}
		onexport={() => (exportOpen = true)}
		ontogglebulle={() => (bulleOpen = !bulleOpen)}
	/>
	<ConnectionBanner {editor} />
	<div class="grid min-h-0 flex-1 gap-3 lg:grid-cols-[13rem_minmax(0,1fr)_16rem]">
		<SlideRail {editor} {assets} />
		<div class="flex min-h-0 flex-col gap-3 max-lg:min-h-[60vh]">
			{#if previewing}
				<div class="min-h-0 flex-1 overflow-hidden rounded-fc-lg bg-fc-component">
					<PreviewPane src={backend.frameUrl(id, { stamp })} {ratio} {playhead} />
				</div>
			{:else}
				<Toolbar {editor} {assets} onfiles={() => (assetsOpen = true)} />
				<div class="relative min-h-0 flex-1 overflow-hidden rounded-fc-lg bg-fc-component">
					<Stage {editor} src={assetUrl} />
					{#if !editor.synced || editor.stopped}
						<div
							class="absolute inset-0 flex items-center justify-center gap-2 bg-fc-component/70 text-fc-sm text-fc-fg-muted"
							data-testid="read-only"
						>
							{#if !editor.stopped}<Spinner />{t('room.syncing')}{/if}
						</div>
					{/if}
				</div>
				<NotesPanel {editor} {user} bind:open={notesOpen} />
			{/if}
		</div>
		<Inspector {editor} />
	</div>
</div>

<AssetsPanel bind:open={assetsOpen} {id} bind:assets oninsert={place} />
<ShareDialog bind:open={shareOpen} {id} {title} width={editor.size.width} height={editor.size.height} />
<ExportDialog bind:open={exportOpen} {id} />
<BulleSidebar
	bind:open={bulleOpen}
	{assistant}
	page={{ app: 'diapo', id, title, url: `${page.url.origin}/decks/${id}` }}
	onready={(name) => (assistant = name)}
/>
