<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import { apply, common } from './inspect';
	import NumberField from './NumberField.svelte';

	let { editor, shapes, disabled }: { editor: DeckEditor; shapes: Shape<FullType>[]; disabled: boolean } = $props();

	const FIELDS = [
		{ key: 'x', label: 'inspect.x' },
		{ key: 'y', label: 'inspect.y' },
		{ key: 'w', label: 'inspect.w' },
		{ key: 'h', label: 'inspect.h' }
	] as const;

	const rotation = $derived(common(shapes, (shape) => shape.rot));
</script>

<section class="flex flex-col gap-2" aria-labelledby="inspect-place" data-testid="inspect-box">
	<h3 id="inspect-place" class="text-fc-xs font-semibold text-fc-fg-muted">{t('inspect.place')}</h3>
	<div class="grid grid-cols-2 gap-2">
	{#each FIELDS as field (field.key)}
		<NumberField
			label={t(field.label)}
			value={common(shapes, (shape) => shape[field.key])}
			min={field.key === 'w' || field.key === 'h' ? 1 : undefined}
			{disabled}
			onchange={(value) => apply(editor, shapes, { [field.key]: value })}
		/>
	{/each}
	<NumberField
		label={t('inspect.rotation')}
		value={rotation === null ? null : (rotation * 180) / Math.PI}
		{disabled}
		onchange={(degrees) => apply(editor, shapes, { rot: (degrees * Math.PI) / 180 })}
	/>
	<NumberField
		label={t('inspect.opacity')}
		value={common(shapes, (shape) => shape.opacity ?? 1)}
		min={0}
		max={1}
		step={0.05}
		{disabled}
		onchange={(opacity) => apply(editor, shapes, { opacity })}
	/>
	</div>
</section>
