<script lang="ts">
	import { TEXT_TYPES, type FullType, type Shape, type ShapeType } from '@facile/planche';
	import type { EditorActions, HAlign, VAlign } from '$lib/editor/actions';
	import { defaultAlign, defaultValign } from '$lib/editor/css';
	import { t } from '$lib/i18n';
	import { common, restyle } from './inspect';
	import NumberField from './NumberField.svelte';
	import SegmentedIcons from './SegmentedIcons.svelte';
	import { tools } from './tools';

	let { actions, shapes, disabled }: { actions: EditorActions; shapes: Shape<FullType>[]; disabled: boolean } =
		$props();

	const worded = $derived(shapes.filter((shape) => TEXT_TYPES.has(shape.type as ShapeType)));
	const across = $derived(
		(['left', 'center', 'right', 'justify'] as const).map((value) => ({
			value,
			icon: tools[`align-${value}`],
			label: t(`align.${value}`)
		}))
	);
	const down = $derived(
		(['top', 'middle', 'bottom'] as const).map((value) => ({
			value,
			icon: tools[`valign-${value}`],
			label: t(`valign.${value}`)
		}))
	);
	const align = $derived(common<HAlign>(worded, (shape) => shape.style.align ?? defaultAlign(shape.type)));
	const valign = $derived(common<VAlign>(worded, (shape) => shape.style.valign ?? defaultValign(shape.type)));
</script>

{#if worded.length}
	<section class="flex flex-col gap-2" aria-labelledby="inspect-text" data-testid="inspect-text">
		<h3 id="inspect-text" class="text-fc-xs font-semibold text-fc-fg-muted">{t('inspect.text')}</h3>
		<NumberField
			label={t('inspect.size')}
			value={common(worded, (shape) => shape.style.size)}
			min={1}
			{disabled}
			onchange={(size) => restyle(actions.editor, worded, { size })}
		/>
		<SegmentedIcons
			label={t('inspect.align')}
			options={across}
			value={align}
			{disabled}
			data-testid="text-align"
			onchange={(value) => actions.textAlign({ align: value })}
		/>
		<SegmentedIcons
			label={t('inspect.valign')}
			options={down}
			value={valign}
			{disabled}
			data-testid="text-valign"
			onchange={(value) => actions.textAlign({ valign: value })}
		/>
	</section>
{/if}
