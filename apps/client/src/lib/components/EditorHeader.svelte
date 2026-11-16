<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Button, Icon, IconButton, Menu, icons, type MenuEntry } from '@facile/muse';
	import { shareIcon } from '$lib/brand';
	import type { SaveStatus } from '$lib/deck/saver.svelte';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';

	let {
		id,
		title,
		status,
		inserts,
		assistant,
		bulleOpen,
		onrename,
		onassets,
		onshare,
		onexport,
		ontogglebulle
	}: {
		id: string;
		title: string;
		status: SaveStatus;
		/** The ready-made scenes of the insert menu. */
		inserts: MenuEntry[];
		/** The name of the person's Bulle assistant. */
		assistant: string;
		bulleOpen: boolean;
		onrename: (title: string) => void;
		onassets: () => void;
		onshare: () => void;
		onexport: () => void;
		ontogglebulle: () => void;
	} = $props();

	const STATUS: Record<SaveStatus, AppKey> = {
		clean: 'save.clean',
		dirty: 'save.dirty',
		saving: 'save.saving',
		invalid: 'save.invalid',
		conflict: 'save.conflict',
		failed: 'save.failed'
	};

	const DOT: Record<SaveStatus, string> = {
		clean: 'bg-fc-success',
		dirty: 'bg-fc-fg-muted',
		saving: 'bg-fc-fg-muted motion-safe:animate-pulse',
		invalid: 'bg-fc-danger',
		conflict: 'bg-fc-warning',
		failed: 'bg-fc-danger'
	};

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
	<span
		class="relative flex items-center gap-2 px-2 text-fc-xs whitespace-nowrap text-fc-fg-muted"
		role="status"
		data-testid="save-status"
		data-status={status}
	>
		<span class="size-2 shrink-0 rounded-fc-full {DOT[status]}" aria-hidden="true"></span>
		<span class="max-sm:sr-only">{t(STATUS[status])}</span>
	</span>
	<div class="ml-auto flex shrink-0 items-center gap-1">
		<Menu label={t('insert.label')} items={inserts} align="end">
			{#snippet trigger()}
				<span class="flex items-center gap-2">
					<Icon icon={icons.plus} size={16} /><span class="hidden md:inline">{t('insert.label')}</span>
				</span>
			{/snippet}
		</Menu>
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
