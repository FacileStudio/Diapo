<script lang="ts">
	import { ContextMenu, Icon, IconButton, icons, type MenuEntry } from '@facile/muse';
	import type { FullType, Shape } from '@facile/planche';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import type { Asset } from '$lib/types';
	import SlideThumb from './SlideThumb.svelte';
	import { plainText } from './text';

	let { editor, assets }: { editor: DeckEditor; assets: Asset[] } = $props();

	let dragged = $state<string | null>(null);
	let edits = $state(0);

	const urls = $derived(new Map(assets.map((asset) => [asset.name, asset.url])));
	const locked = $derived(!editor.synced || editor.stopped !== null);
	const text = (shape: Shape<FullType>) => (edits < 0 ? '' : (shape.text ?? plainText(editor.text(shape.id))));

	$effect(() => {
		const ydoc = editor.doc.ydoc;
		const bump = () => (edits += 1);
		ydoc.on('update', bump);
		return () => ydoc.off('update', bump);
	});

	function add(after?: string) {
		editor.show(editor.addSlide(after));
	}

	function entries(target: Element): MenuEntry[] {
		const id = target.closest<HTMLElement>('[data-slide]')?.dataset.slide;
		if (!id || locked) return [];
		return [
			{ label: t('rail.addAfter'), icon: icons.plus, onselect: () => add(id) },
			{ label: t('rail.duplicate'), icon: icons.copy, onselect: () => editor.show(editor.duplicateSlide(id)) },
			{ separator: true },
			{
				label: t('rail.delete'),
				icon: icons.remove,
				tone: 'danger',
				disabled: editor.slides.length < 2,
				onselect: () => editor.removeSlide(id)
			}
		];
	}

	function drop(event: DragEvent, target: string) {
		event.preventDefault();
		const id = dragged;
		dragged = null;
		if (!id || id === target) return;
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const index = editor.slides.findIndex((slide) => slide.id === target);
		const below = event.clientY > box.top + box.height / 2;
		editor.moveSlide(id, below ? (editor.slides[index + 1]?.id ?? null) : target);
	}

	function onkeydown(event: KeyboardEvent, index: number) {
		if (!event.altKey || locked || (event.key !== 'ArrowUp' && event.key !== 'ArrowDown')) return;
		event.preventDefault();
		const id = editor.slides[index].id;
		const before = event.key === 'ArrowUp' ? editor.slides[index - 1] : editor.slides[index + 2];
		if (event.key === 'ArrowUp' && !before) return;
		editor.moveSlide(id, before?.id ?? null);
	}
</script>

<nav class="flex min-h-0 flex-col gap-2 rounded-fc-lg bg-fc-component p-2" aria-label={t('rail.title')}>
	<div class="flex items-center gap-1 pl-1.5">
		<p class="min-w-0 flex-1 truncate text-fc-xs text-fc-fg-muted" data-testid="slide-count">
			{t('rail.count', { count: editor.slides.length })}
		</p>
		<IconButton
			variant="ghost"
			aria-label={t('rail.add')}
			disabled={locked}
			data-testid="slide-add"
			onclick={() => add(editor.current ?? undefined)}
		>
			<Icon icon={icons.plus} size={18} />
		</IconButton>
	</div>
	<ContextMenu items={entries} label={t('rail.menu')} class="min-h-0 flex-1 overflow-y-auto">
		<ol class="flex flex-col gap-1" data-testid="slide-rail">
			{#each editor.slides as slide, index (slide.id)}
				<li data-slide={slide.id} ondragover={(event) => event.preventDefault()} ondrop={(event) => drop(event, slide.id)}>
					<button
						type="button"
						class="flex w-full items-start gap-2 rounded-fc-md p-1.5 text-left hover:bg-fc-surface
							focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring
							{slide.id === editor.current ? 'bg-fc-surface' : ''} {dragged === slide.id ? 'opacity-50' : ''}"
						aria-current={slide.id === editor.current ? 'true' : undefined}
						aria-label={slide.name || t('rail.slide', { number: index + 1 })}
						draggable={!locked}
						data-testid="slide-thumb"
						ondragstart={() => (dragged = slide.id)}
						ondragend={() => (dragged = null)}
						onkeydown={(event) => onkeydown(event, index)}
						onclick={() => editor.show(slide.id)}
					>
						<span class="w-5 shrink-0 pt-0.5 text-right text-fc-xs text-fc-fg-muted tabular-nums">{index + 1}</span>
						<span class="min-w-0 flex-1"><SlideThumb {slide} size={editor.size} {urls} {text} /></span>
					</button>
				</li>
			{/each}
		</ol>
	</ContextMenu>
</nav>
