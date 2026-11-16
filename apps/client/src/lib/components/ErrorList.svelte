<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { t } from '$lib/i18n';
	import type { SceneError } from '$lib/types';

	let { errors, onpick }: { errors: SceneError[]; onpick: (line: number) => void } = $props();
</script>

<ul class="flex max-h-32 flex-col gap-1 overflow-auto p-2" aria-label={t('errors.title')} data-testid="scene-errors">
	{#each errors as error (`${error.line}:${error.message}`)}
		<li>
			<button
				type="button"
				class="flex min-h-11 w-full items-center gap-2 rounded-fc-md bg-fc-danger/10 px-3 py-2 text-left text-fc-sm
					text-fc-danger focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
				onclick={() => onpick(error.line)}
			>
				<Icon icon={icons.warning} size={16} class="shrink-0" />
				<span class="min-w-0"
					><b class="font-semibold">{t('errors.line', { line: error.line })}</b> {error.message}</span
				>
			</button>
		</li>
	{/each}
</ul>
