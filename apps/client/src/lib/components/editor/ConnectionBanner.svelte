<script lang="ts">
	import { Alert, Button } from '@facile/muse';
	import type { Stop } from '@facile/salon';
	import type { DeckEditor } from '$lib/editor/types';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';

	let { editor }: { editor: DeckEditor } = $props();

	const STOPS: Record<Stop, AppKey> = { refused: 'room.refused', 'too-large': 'room.too-large', failed: 'room.failed' };

	const reconnecting = $derived(editor.connection === 'connecting' && editor.synced);
	const waiting = $derived<AppKey | null>(
		editor.connection === 'offline' ? 'room.offline' : reconnecting ? 'room.connecting' : null
	);
</script>

{#if editor.stopped}
	<Alert tone="danger" data-testid="room-stopped" data-stop={editor.stopped.stop}>
		{t(STOPS[editor.stopped.stop])}
		{#snippet action()}
			<Button size="sm" variant="secondary" onclick={() => location.reload()}>{t('room.reload')}</Button>
		{/snippet}
	</Alert>
{:else if waiting}
	<Alert tone="warning" data-testid="room-waiting" data-connection={editor.connection}>
		{t(waiting)}
	</Alert>
{/if}
