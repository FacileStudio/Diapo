<script lang="ts">
	import { Editor, type CollabUser, type Locale } from '@facile/encre';
	import { Icon, icons } from '@facile/muse';
	import type { DeckEditor } from '$lib/editor/types';
	import { locale, t } from '$lib/i18n';

	let {
		editor,
		user,
		open = $bindable()
	}: {
		editor: DeckEditor;
		/** This person, as the carets of other people show them. */
		user: CollabUser;
		open: boolean;
	} = $props();

	const slide = $derived(editor.current);
	const editable = $derived(editor.synced && editor.stopped === null);
	const language = locale() as Locale;
</script>

<section
	class="flex shrink-0 flex-col rounded-fc-lg bg-fc-component {open ? 'h-48' : ''}"
	aria-label={t('notes.title')}
	data-testid="notes-panel"
	data-open={open}
>
	<button
		type="button"
		class="flex min-h-11 items-center gap-2 rounded-fc-lg px-3 text-left text-fc-sm font-semibold text-fc-fg
			hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
		aria-expanded={open}
		aria-label={open ? t('notes.hide') : t('notes.show')}
		onclick={() => (open = !open)}
	>
		<Icon icon={open ? icons.chevronDown : icons.chevronUp} size={16} />
		{t('notes.title')}
	</button>
	{#if open && slide}
		<div class="min-h-0 flex-1 overflow-y-auto px-3 pb-3" data-testid="notes-editor">
			{#key slide}
				<Editor
					fragment={editor.notes(slide)}
					provider={editor.provider ?? undefined}
					{user}
					{editable}
					presence="none"
					toolbar={false}
					locale={language}
					placeholder={t('notes.placeholder')}
				/>
			{/key}
		</div>
	{/if}
</section>
