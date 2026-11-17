<script lang="ts">
	import { Input, Switch } from '@facile/muse';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import { apply, common } from './inspect';
	import InspectorBox from './InspectorBox.svelte';
	import InspectorStyle from './InspectorStyle.svelte';
	import NumberField from './NumberField.svelte';

	let { editor }: { editor: DeckEditor } = $props();

	const slide = $derived(editor.slides.find((found) => found.id === editor.current) ?? null);
	const shapes = $derived(
		editor.slides.flatMap((found) => found.shapes).filter((shape) => editor.selection.includes(shape.id))
	);
	const disabled = $derived(!editor.synced || editor.stopped !== null);
	const locked = $derived(common(shapes, (shape) => shape.locked ?? false));

	function rename(name: string) {
		const next = name.trim();
		if (slide && next !== slide.name) editor.update(slide.id, { name: next });
	}
</script>

<aside
	class="flex min-h-0 flex-col gap-4 overflow-y-auto rounded-fc-lg bg-fc-component p-3"
	aria-label={t('inspect.title')}
	data-testid="inspector"
	data-count={shapes.length}
>
	{#if shapes.length === 0}
		<h2 class="text-fc-sm font-semibold text-fc-fg">{t('inspect.slide')}</h2>
		{#if slide}
			<label class="flex flex-col gap-1 text-fc-xs text-fc-fg-muted">
				{t('inspect.name')}
				{#key slide.id}
					<Input
						value={slide.name}
						maxlength={200}
						{disabled}
						data-testid="slide-name"
						onchange={(event) => rename(event.currentTarget.value)}
					/>
				{/key}
			</label>
			<p class="flex justify-between gap-2 text-fc-xs text-fc-fg-muted">
				<span>{t('inspect.layout')}</span><span class="text-fc-fg">{t('inspect.layout.none')}</span>
			</p>
		{/if}
	{:else}
		<h2 class="text-fc-sm font-semibold text-fc-fg">{t('inspect.count', { count: shapes.length })}</h2>
		<InspectorBox {editor} {shapes} {disabled} />
		<InspectorStyle {editor} {shapes} {disabled} />
		<NumberField
			label={t('inspect.step')}
			value={common(shapes, (shape) => shape.step ?? 0)}
			min={0}
			max={1000}
			{disabled}
			onchange={(step) => apply(editor, shapes, { step: Math.round(step) })}
		/>
		<Switch
			label={t('inspect.lock')}
			checked={locked ?? false}
			{disabled}
			onchange={(event) => apply(editor, shapes, { locked: event.currentTarget.checked })}
		/>
	{/if}
</aside>
