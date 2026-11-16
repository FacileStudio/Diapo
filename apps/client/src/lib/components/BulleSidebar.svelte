<script lang="ts">
	import { untrack } from 'svelte';
	import { bulleMessage, embedUrl, trustedOrigin, type PageContext } from '$lib/bulle';
	import { session } from '$lib/session.svelte';

	let {
		open = $bindable(),
		page,
		assistant,
		onready
	}: {
		open: boolean;
		page: PageContext;
		assistant: string;
		/** Bulle named the assistant once its frame loaded. */
		onready?: (name: string) => void;
	} = $props();

	let frame = $state<HTMLIFrameElement | null>(null);
	let shown = $state(false);

	const src = $derived.by(() => {
		const id = page.id;
		const origin = session.bulle;
		return untrack(() => embedUrl({ ...page, id }, origin));
	});
	const dev = import.meta.env.DEV;

	$effect(() => {
		if (open) shown = true;
	});

	function onmessage(event: MessageEvent) {
		if (!trustedOrigin(event.origin, session.bulle, dev) || event.source !== frame?.contentWindow) return;
		const message = bulleMessage(event.data);
		if (message?.type === 'bulle:close') open = false;
		if (message?.type === 'bulle:ready') onready?.(message.assistant.name);
	}
</script>

<svelte:window
	{onmessage}
	onkeydown={(event) => {
		if (open && event.key === 'Escape' && !event.defaultPrevented) open = false;
	}}
/>

{#if shown}
	<aside
		class="fixed inset-0 z-40 flex flex-col bg-fc-bg shadow-lg lg:inset-y-0 lg:right-0 lg:left-auto lg:w-[400px]"
		class:hidden={!open}
		aria-label={assistant}
		data-testid="bulle-sidebar"
	>
		<iframe bind:this={frame} title={assistant} {src} class="min-h-0 w-full flex-1 bg-fc-bg" allow="clipboard-write"
		></iframe>
	</aside>
{/if}
