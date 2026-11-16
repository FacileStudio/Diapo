<script lang="ts">
	import { onDestroy, untrack } from 'svelte';
	import { page } from '$app/state';
	import { Tabs, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { assistant as askAssistant, DEFAULT_ASSISTANT, rememberSidebar, sidebarOpen } from '$lib/bulle';
	import { watchDeck } from '$lib/deck/events';
	import { outline, sceneAt, type SceneOutline } from '$lib/deck/outline';
	import { Playhead } from '$lib/deck/playhead.svelte';
	import { Saver } from '$lib/deck/saver.svelte';
	import { PLACEHOLDER_IMAGE, TEMPLATES, type Template } from '$lib/deck/templates';
	import { t } from '$lib/i18n';
	import { session } from '$lib/session.svelte';
	import type { Asset, Deck } from '$lib/types';
	import AssetsPanel from './AssetsPanel.svelte';
	import BulleSidebar from './BulleSidebar.svelte';
	import ConflictBanner from './ConflictBanner.svelte';
	import EditorHeader from './EditorHeader.svelte';
	import ErrorList from './ErrorList.svelte';
	import PreviewPane from './PreviewPane.svelte';
	import SceneStrip from './SceneStrip.svelte';
	import SourceEditor from './SourceEditor.svelte';

	let { deck }: { deck: Deck } = $props();

	const id = untrack(() => deck.id);
	const owner = session.user?.email ?? '';
	const playhead = new Playhead();
	const saver = new Saver({
		put: (source, rev) => backend.save(id, source, rev),
		get: () => backend.deck(id),
		onreplace: (source) => {
			editor?.replace(source);
			text = source;
		}
	});
	saver.open(untrack(() => deck));

	let editor = $state<ReturnType<typeof SourceEditor>>();
	let title = $state(untrack(() => deck.title));
	let text = $state(untrack(() => deck.source));
	let line = $state(1);
	let tab = $state('source');
	let touches = $state(0);
	let stripStamp = $state(`${saver.rev}.0`);
	let assets = $state<Asset[]>([]);
	let assetsOpen = $state(false);
	let assistant = $state(DEFAULT_ASSISTANT.name);
	let bulleOpen = $state(sidebarOpen(owner));

	const plan = $derived(outline(text));
	const scene = $derived(sceneAt(plan.scenes, line));
	const ratio = $derived(plan.width / plan.height);
	const loadAssets = () => backend.assets(id).then((found) => (assets = found));
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	$effect(() => {
		const index = scene;
		untrack(() => playhead.focus(index));
	});

	const stamp = $derived(`${saver.rev}.${touches}`);

	$effect(() => {
		const next = stamp;
		const timer = setTimeout(() => (stripStamp = next), 1200);
		return () => clearTimeout(timer);
	});

	$effect(() => rememberSidebar(owner, bulleOpen));

	$effect(() => watchDeck(id, changed));

	void askAssistant().then((found) => (assistant = found.name));
	void loadAssets().catch(() => undefined);

	onDestroy(() => {
		void saver.flush();
		saver.dispose();
	});

	async function refresh() {
		try {
			await loadAssets();
			title = (await backend.deck(id)).title;
		} catch {
			return;
		}
	}

	function changed(rev: number, first: boolean) {
		if (!first && saver.touched(rev)) {
			touches += 1;
			void refresh();
		}
		void saver.remote(rev).catch(failed);
	}

	function edited(source: string) {
		text = source;
		saver.edit(source);
	}

	async function rename(next: string) {
		try {
			title = (await backend.rename(id, next)).title;
		} catch (error) {
			failed(error);
		}
	}

	function insert(template: Template) {
		const image = assets.find((asset) => asset.mime.startsWith('image/'));
		const body = template.scene(image?.name ?? PLACEHOLDER_IMAGE);
		editor?.insertAfter(plan.scenes[scene]?.to ?? 1, `\n${body}`);
	}

	function pick(target: SceneOutline) {
		tab = 'source';
		editor?.reveal(Math.min(target.from + 1, target.to));
	}

	const inserts = $derived(
		TEMPLATES.map((template) => ({ label: t(template.label), onselect: () => insert(template) }))
	);
	const tabs = $derived([
		{ id: 'source', label: t('editor.source') },
		{ id: 'preview', label: t('preview.title') }
	]);
</script>

<svelte:head><title>{title} · Diapo</title></svelte:head>

<svelte:window
	onbeforeunload={(event) => {
		if (saver.status !== 'clean') event.preventDefault();
	}}
/>

<div class="flex h-full min-h-0 flex-col gap-3 p-3 {bulleOpen ? 'lg:pr-[412px]' : ''}">
	<EditorHeader
		{id}
		{title}
		status={saver.status}
		{inserts}
		{assistant}
		{bulleOpen}
		onrename={rename}
		onassets={() => (assetsOpen = true)}
		ontogglebulle={() => (bulleOpen = !bulleOpen)}
	/>
	{#if saver.status === 'conflict'}
		<ConflictBanner
			onreload={() => void saver.reload().catch(failed)}
			onoverwrite={() => void saver.overwrite().catch(failed)}
		/>
	{/if}
	<Tabs items={tabs} value={tab} onChange={(next) => (tab = next)} label={t('editor.views')} class="lg:hidden" />
	<div class="grid min-h-0 flex-1 gap-3 lg:grid-cols-2">
		<div
			class="min-h-0 flex-col overflow-hidden rounded-fc-lg bg-fc-component lg:flex {tab === 'source'
				? 'flex'
				: 'hidden'}"
		>
			<div class="min-h-0 flex-1">
				<SourceEditor
					bind:this={editor}
					value={deck.source}
					errors={saver.errors}
					label={t('editor.source')}
					onchange={edited}
					oncursor={(next) => (line = next)}
				/>
			</div>
			{#if saver.errors.length > 0}
				<ErrorList errors={saver.errors} onpick={(target) => editor?.reveal(target)} />
			{/if}
		</div>
		<div
			class="min-h-0 overflow-hidden rounded-fc-lg bg-fc-component lg:block {tab === 'preview' ? 'block' : 'hidden'}"
		>
			<PreviewPane src={backend.frameUrl(id, { stamp })} {ratio} {playhead} />
		</div>
	</div>
	<SceneStrip
		{id}
		stamp={stripStamp}
		scenes={plan.scenes}
		compiled={playhead.info?.scenes.length ?? 0}
		{ratio}
		current={scene}
		onpick={pick}
	/>
</div>

<AssetsPanel bind:open={assetsOpen} {id} bind:assets />
<BulleSidebar
	bind:open={bulleOpen}
	{assistant}
	page={{ app: 'diapo', id, title, url: `${page.url.origin}/decks/${id}` }}
	onready={(name) => (assistant = name)}
/>
