<script lang="ts">
	import { Alert, Button } from '@facile/muse';
	import { t } from '$lib/i18n';

	let { onreload, onoverwrite }: { onreload: () => void; onoverwrite: () => void } = $props();
</script>

<Alert tone="warning" title={t('conflict.title')} data-testid="conflict">
	{t('conflict.body')}
	{#snippet action()}
		<span class="flex flex-wrap gap-2">
			<Button size="sm" variant="secondary" onclick={onreload}>{t('conflict.reload')}</Button>
			<Button size="sm" variant="secondary" onclick={onoverwrite}>{t('conflict.overwrite')}</Button>
		</span>
	{/snippet}
</Alert>
