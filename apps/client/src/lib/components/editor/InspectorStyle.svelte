<script lang="ts">
	import { ColorPicker, Select } from '@facile/muse';
	import { TEXT_TYPES, type FullType, type Shape, type ShapeType, type Style } from '@facile/planche';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import { common, restyle } from './inspect';
	import NumberField from './NumberField.svelte';

	let { editor, shapes, disabled }: { editor: DeckEditor; shapes: Shape<FullType>[]; disabled: boolean } = $props();

	const worded = $derived(shapes.some((shape) => TEXT_TYPES.has(shape.type as ShapeType)));
	const aligns = $derived(
		(['left', 'center', 'right'] as const).map((value) => ({ value, label: t(`align.${value}`) }))
	);
	const style = (patch: Style) => restyle(editor, shapes, patch);
</script>

<div class="flex flex-col gap-3" data-testid="inspect-style">
	<label class="flex flex-col gap-1 text-fc-xs text-fc-fg-muted">
		{t('inspect.fill')}
		<ColorPicker value={common(shapes, (shape) => shape.style.fill) ?? ''} {disabled} onChange={(fill) => style({ fill })} />
	</label>
	<label class="flex flex-col gap-1 text-fc-xs text-fc-fg-muted">
		{t('inspect.stroke')}
		<ColorPicker
			value={common(shapes, (shape) => shape.style.stroke) ?? ''}
			{disabled}
			onChange={(stroke) => style({ stroke })}
		/>
	</label>
	<div class="grid grid-cols-2 gap-2">
		<NumberField
			label={t('inspect.strokeWidth')}
			value={common(shapes, (shape) => shape.style.width ?? 0)}
			min={0}
			{disabled}
			onchange={(width) => style({ width })}
		/>
		{#if worded}
			<NumberField
				label={t('inspect.size')}
				value={common(shapes, (shape) => shape.style.size)}
				min={1}
				{disabled}
				onchange={(size) => style({ size })}
			/>
		{/if}
	</div>
	{#if worded}
		<label class="flex flex-col gap-1 text-fc-xs text-fc-fg-muted">
			{t('inspect.align')}
			<Select
				value={common(shapes, (shape) => shape.style.align ?? 'left') ?? ''}
				options={aligns}
				placeholder={t('inspect.mixed')}
				{disabled}
				onchange={(event) => style({ align: event.currentTarget.value as Style['align'] })}
			/>
		</label>
	{/if}
</div>
