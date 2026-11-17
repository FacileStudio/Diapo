<script lang="ts">
	import { Input } from '@facile/muse';
	import { t } from '$lib/i18n';

	let {
		label,
		value,
		min,
		max,
		step = 1,
		disabled = false,
		onchange
	}: {
		label: string;
		/** The shown number; null when the selected shapes disagree, undefined when none sets it. */
		value: number | null | undefined;
		min?: number;
		max?: number;
		step?: number;
		disabled?: boolean;
		onchange: (value: number) => void;
	} = $props();

	const shown = $derived(
		value === null || value === undefined ? '' : String(Number(value.toFixed(step < 1 ? 2 : 0)))
	);

	function commit(raw: string) {
		const next = Number.parseFloat(raw);
		if (!Number.isFinite(next)) return;
		onchange(Math.min(max ?? Infinity, Math.max(min ?? -Infinity, next)));
	}
</script>

<label class="flex min-w-0 flex-col gap-1 text-fc-xs text-fc-fg-muted">
	{label}
	<Input
		type="number"
		inputmode="decimal"
		value={shown}
		{min}
		{max}
		{step}
		{disabled}
		placeholder={value === null ? t('inspect.mixed') : ''}
		onchange={(event) => commit(event.currentTarget.value)}
	/>
</label>
