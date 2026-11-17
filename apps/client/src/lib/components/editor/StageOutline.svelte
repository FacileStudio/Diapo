<script lang="ts">
	import type { FullType, Shape } from '@facile/planche';

	let {
		shape,
		scale,
		color = 'var(--color-fc-info)'
	}: {
		/** The shape to outline, in frame pixels. */
		shape: Shape<FullType>;
		/** Screen pixels per frame pixel. */
		scale: number;
		/** The outline colour: muse's info blue for this person, a peer's own colour for theirs. */
		color?: string;
	} = $props();
</script>

<div
	class="pointer-events-none absolute top-0 left-0 z-10 origin-center"
	style:width="{shape.w * scale}px"
	style:height="{shape.h * scale}px"
	style:transform="translate({shape.x * scale}px, {shape.y * scale}px) rotate({shape.rot}rad)"
	style:box-shadow="0 0 0 1.5px {color}"
	data-outline={shape.id}
></div>
