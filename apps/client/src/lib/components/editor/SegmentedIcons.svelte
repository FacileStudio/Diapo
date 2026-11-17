<script lang="ts" generics="V extends string">
	import { Icon, Tooltip } from '@facile/muse';

	let {
		label,
		options,
		value,
		disabled = false,
		onchange,
		...rest
	}: {
		/** The group's accessible name. */
		label: string;
		/** The choices, each with its glyph and its name, shown as a tooltip. */
		options: { value: V; icon: string; label: string }[];
		/** The chosen value; null when the selection disagrees. */
		value: V | null;
		disabled?: boolean;
		onchange: (value: V) => void;
		'data-testid'?: string;
	} = $props();

	function onkeydown(event: KeyboardEvent, index: number) {
		const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const next = options[(index + step + options.length) % options.length];
		onchange(next.value);
		const group = (event.currentTarget as HTMLElement).parentElement?.parentElement;
		group?.querySelectorAll<HTMLElement>('[role="radio"]')[options.indexOf(next)]?.focus();
	}
</script>

<div class="flex gap-0.5 rounded-fc-md bg-fc-surface p-0.5" role="radiogroup" aria-label={label} {...rest}>
	{#each options as option, index (option.value)}
		{@const checked = option.value === value}
		<Tooltip text={option.label} describe={false} class="flex flex-1">
			<button
				type="button"
				role="radio"
				aria-checked={checked}
				aria-label={option.label}
				tabindex={checked || (value === null && index === 0) ? 0 : -1}
				{disabled}
				data-value={option.value}
				class="flex min-h-9 flex-1 items-center justify-center rounded-fc-sm text-fc-fg-muted transition-colors
					hover:text-fc-fg focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-fc-ring
					disabled:cursor-not-allowed disabled:opacity-50 {checked ? 'bg-fc-component text-fc-fg shadow-sm' : ''}"
				onclick={() => onchange(option.value)}
				onkeydown={(event) => onkeydown(event, index)}
			>
				<Icon icon={option.icon} size={16} />
			</button>
		</Tooltip>
	{/each}
</div>
