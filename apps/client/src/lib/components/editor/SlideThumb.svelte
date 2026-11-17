<script lang="ts">
	import { compareZ, type FullType, type Shape } from '@facile/planche';
	import type { Slide } from '$lib/editor/types';

	let {
		slide,
		size,
		urls,
		text
	}: {
		slide: Slide;
		/** The stage size in pixels: shape coordinates are frame pixels. */
		size: { width: number; height: number };
		/** Asset name to its address, for image shapes. */
		urls: Map<string, string>;
		/** The plain text of a shape, for text-bearing shapes. */
		text: (shape: Shape<FullType>) => string;
	} = $props();

	const CLIP: Partial<Record<FullType, string>> = {
		ellipse: 'border-radius: 50%;',
		diamond: 'clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%);'
	};
	const FILLED = new Set<FullType>(['rect', 'ellipse', 'diamond', 'sticky']);

	const shapes = $derived(slide.shapes.filter((shape) => shape.type !== 'audio').sort(compareZ));

	function place(shape: Shape<FullType>): string {
		const pct = (value: number, of: number) => `${(value / of) * 100}%`;
		const box = `left:${pct(shape.x, size.width)};top:${pct(shape.y, size.height)};`;
		const area = `width:${pct(shape.w, size.width)};height:${pct(shape.h, size.height)};`;
		const turn = shape.rot ? `transform:rotate(${shape.rot}rad);` : '';
		return box + area + turn + `opacity:${shape.opacity ?? 1};`;
	}

	function paint(shape: Shape<FullType>): string {
		const fill = shape.style.fill ?? (FILLED.has(shape.type) ? 'var(--color-fc-surface)' : 'transparent');
		const line = shape.style.stroke ? `box-shadow:inset 0 0 0 ${shape.style.width ?? 2}px ${shape.style.stroke};` : '';
		const font = `font-size:${((shape.style.size ?? 32) / size.width) * 100}cqw;text-align:${shape.style.align ?? 'left'};`;
		return `background:${fill};${line}${font}${CLIP[shape.type] ?? ''}`;
	}
</script>

<span
	class="relative block w-full overflow-hidden rounded-fc-sm bg-white text-black shadow-sm [container-type:inline-size]"
	style="aspect-ratio: {size.width} / {size.height}"
	aria-hidden="true"
>
	{#each shapes as shape (shape.id)}
		<span class="absolute overflow-hidden leading-tight whitespace-pre-line" style={place(shape) + paint(shape)}>
			{#if shape.type === 'image' && shape.asset && urls.get(shape.asset)}
				<img class="size-full object-cover" src={urls.get(shape.asset)} alt="" loading="lazy" />
			{:else if shape.type === 'video'}
				<span class="block size-full bg-black/80"></span>
			{:else if shape.type === 'html'}
				<span class="block size-full bg-black/10"></span>
			{:else}
				{text(shape)}
			{/if}
		</span>
	{/each}
</span>
