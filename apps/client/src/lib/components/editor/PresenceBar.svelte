<script lang="ts">
	import { Presence } from '@facile/muse';
	import type { DeckEditor } from '$lib/editor/types';

	let { editor }: { editor: DeckEditor } = $props();

	const peers = $derived(editor.peers.map((peer) => ({ id: peer.client, name: peer.name, color: peer.color })));

	function follow(client: string | number | null) {
		const slide = editor.peers.find((peer) => peer.client === client)?.slide;
		if (slide) editor.show(slide);
	}
</script>

{#if peers.length > 0}
	<Presence {peers} onfollow={follow} data-testid="presence" />
{/if}
