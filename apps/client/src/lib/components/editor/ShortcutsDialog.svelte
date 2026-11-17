<script lang="ts">
	import { Kbd, Modal } from '@facile/muse';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';

	let { open = $bindable() }: { open: boolean } = $props();

	const ROWS: [AppKey, string[]][] = [
		['keys.edit', ['enter']],
		['keys.leave', ['esc']],
		['keys.nudge', ['left', 'shift+left']],
		['menu.copy', ['mod+c']],
		['menu.cut', ['mod+x']],
		['menu.paste', ['mod+v']],
		['menu.duplicate', ['mod+d']],
		['menu.selectAll', ['mod+a']],
		['menu.delete', ['delete', 'backspace']],
		['tool.undo', ['mod+z']],
		['tool.redo', ['mod+shift+z', 'mod+y']],
		['menu.forward', ['mod+]']],
		['menu.backward', ['mod+[']],
		['tool.front', ['mod+shift+]']],
		['tool.back', ['mod+shift+[']],
		['keys.textAlign', ['mod+shift+l', 'mod+shift+e', 'mod+shift+r', 'mod+shift+j']],
		['keys.rail', ['alt+up', 'alt+down']],
		['keys.menu', ['shift+F10']],
		['keys.addText', []],
		['menu.shortcuts', ['?']]
	];
</script>

<Modal {open} title={t('menu.shortcuts')} onClose={() => (open = false)}>
	<dl class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-2 text-fc-sm" data-testid="shortcuts">
		{#each ROWS as [label, keys] (label)}
			<dt class="text-fc-fg">{t(label)}</dt>
			<dd class="flex flex-wrap justify-end gap-1.5 text-fc-fg-muted">
				{#if keys.length}
					{#each keys as combo (combo)}<Kbd keys={combo} />{/each}
				{:else}
					{t('keys.doubleClick')}
				{/if}
			</dd>
		{/each}
	</dl>
</Modal>
