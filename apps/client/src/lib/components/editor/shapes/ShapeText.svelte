<script lang="ts">
	import { Editor } from '@facile/encre';
	import type { FullType, Shape } from '@facile/planche';
	import type { DeckEditor } from '$lib/editor/types';

	let { shape, editor }: { shape: Shape<FullType>; editor: DeckEditor } = $props();

	const fragment = $derived(editor.text(shape.id));
	const editing = $derived(editor.editing === shape.id);
	const user = $derived.by(() => {
		const local = editor.awareness.getLocalState()?.user as { name?: string; color?: string } | undefined;
		return { name: local?.name ?? '', color: local?.color };
	});
</script>

{#if fragment}
	{#key fragment}
		<Editor
			{fragment}
			provider={editor.provider ?? undefined}
			{user}
			editable={editing}
			toolbar={false}
			presence="none"
			class="stage-text"
		/>
	{/key}
{:else if shape.text}
	<p class="whitespace-pre-wrap">{shape.text}</p>
{/if}
