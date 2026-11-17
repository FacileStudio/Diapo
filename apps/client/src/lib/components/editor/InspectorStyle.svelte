<script lang="ts">
	import { ColorPicker } from '@facile/muse';
	import type { FullType, Shape, Style } from '@facile/planche';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import { common, restyle } from './inspect';
	import NumberField from './NumberField.svelte';

	let { editor, shapes, disabled }: { editor: DeckEditor; shapes: Shape<FullType>[]; disabled: boolean } = $props();

	const style = (patch: Style) => restyle(editor, shapes, patch);
</script>

<section class="flex flex-col gap-2" aria-labelledby="inspect-look" data-testid="inspect-style">
	<h3 id="inspect-look" class="text-fc-xs font-semibold text-fc-fg-muted">{t('inspect.look')}</h3>
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
	<NumberField
		label={t('inspect.strokeWidth')}
		value={common(shapes, (shape) => shape.style.width ?? 0)}
		min={0}
		{disabled}
		onchange={(width) => style({ width })}
	/>
</section>
