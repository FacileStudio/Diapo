<script lang="ts">
	import { untrack } from 'svelte';
	import { stageMessage, type StageInfo, type StageState } from '$lib/deck/stage';

	type Layer = { key: number; src: string; phase: 'loading' | 'stepping' | 'shown' };

	let {
		src,
		title,
		step = 0,
		lazy = false,
		onready,
		onstate
	}: {
		/** The URL of a compiled document. A new URL loads behind the shown one and replaces it once drawn. */
		src: string;
		title: string;
		/** The global step to draw. */
		step?: number;
		lazy?: boolean;
		onready?: (info: StageInfo) => void;
		onstate?: (state: StageState) => void;
	} = $props();

	let layers = $state<Layer[]>([]);
	let serial = 0;
	const frames = new Map<number, HTMLIFrameElement>();

	$effect(() => {
		const next = src;
		untrack(() => {
			const shown = layers.filter((layer) => layer.phase === 'shown').slice(-1);
			layers = [...shown, { key: (serial += 1), src: next, phase: 'loading' }];
		});
	});

	$effect(() => {
		const k = step;
		for (const layer of layers) {
			if (layer.phase !== 'loading') frames.get(layer.key)?.contentWindow?.postMessage({ type: 'scene:step', k }, '*');
		}
	});

	function register(node: HTMLIFrameElement, key: number) {
		frames.set(key, node);
		return { destroy: () => frames.delete(key) };
	}

	function onmessage(event: MessageEvent) {
		const layer = layers.find((known) => frames.get(known.key)?.contentWindow === event.source);
		const message = layer ? stageMessage(event.data) : null;
		if (!layer || !message) return;
		if (message.type === 'scene:ready') {
			layer.phase = 'stepping';
			onready?.(message);
			return;
		}
		if (layer.phase === 'loading') return;
		layer.phase = 'shown';
		layers = layers.filter((known) => known.key >= layer.key);
		onstate?.(message);
	}
</script>

<svelte:window {onmessage} />

<div class="relative size-full overflow-hidden bg-fc-surface">
	{#each layers as layer (layer.key)}
		<iframe
			use:register={layer.key}
			src={layer.src}
			{title}
			sandbox="allow-scripts"
			referrerpolicy="no-referrer"
			loading={lazy ? 'lazy' : 'eager'}
			tabindex="-1"
			class="pointer-events-none absolute inset-0 size-full {layer.phase === 'shown' ? '' : 'opacity-0'}"
		></iframe>
	{/each}
</div>
