<script lang="ts">
	import { basicSetup } from 'codemirror';
	import { indentWithTab } from '@codemirror/commands';
	import { markdown } from '@codemirror/lang-markdown';
	import { lintGutter } from '@codemirror/lint';
	import { EditorView, keymap, type ViewUpdate } from '@codemirror/view';
	import { look, markErrors } from '$lib/deck/codemirror';
	import type { SceneError } from '$lib/types';

	let {
		value,
		errors,
		label,
		onchange,
		oncursor
	}: {
		/** The source the editor opens with. Later changes come through `replace`. */
		value: string;
		errors: SceneError[];
		label: string;
		onchange: (source: string) => void;
		/** The caret moved to `line` (from 1). */
		oncursor: (line: number) => void;
	} = $props();

	let view: EditorView | undefined;
	let silent = false;

	function updated(update: ViewUpdate) {
		if (update.docChanged && !silent) onchange(update.state.doc.toString());
		if (update.selectionSet || update.docChanged) {
			oncursor(update.state.doc.lineAt(update.state.selection.main.head).number);
		}
	}

	function mount(node: HTMLElement) {
		view = new EditorView({
			parent: node,
			doc: value,
			extensions: [
				basicSetup,
				keymap.of([indentWithTab]),
				markdown({ extensions: [{ remove: ['SetextHeading'] }] }),
				lintGutter(),
				EditorView.lineWrapping,
				EditorView.contentAttributes.of({ 'aria-label': label }),
				EditorView.updateListener.of(updated),
				look
			]
		});
		return { destroy: () => view?.destroy() };
	}

	$effect(() => {
		const found = errors;
		if (view) view.dispatch(markErrors(view.state, found));
	});

	/** Replaces the whole text without reporting a change. */
	export function replace(source: string): void {
		if (!view) return;
		silent = true;
		view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: source } });
		silent = false;
	}

	/** Inserts `text` after line `line` (from 1), puts the caret at its end and reports the change. */
	export function insertAfter(line: number, text: string): void {
		if (!view) return;
		const at = view.state.doc.line(Math.min(Math.max(line, 1), view.state.doc.lines)).to;
		view.dispatch({
			changes: { from: at, insert: text },
			selection: { anchor: at + text.length },
			scrollIntoView: true
		});
		view.focus();
	}

	/** Puts the caret on line `line` (from 1) and scrolls it into view. */
	export function reveal(line: number): void {
		if (!view) return;
		const at = view.state.doc.line(Math.min(Math.max(line, 1), view.state.doc.lines)).from;
		view.dispatch({ selection: { anchor: at }, effects: EditorView.scrollIntoView(at, { y: 'start', yMargin: 24 }) });
		view.focus();
	}
</script>

<div use:mount class="size-full min-h-0 overflow-hidden" data-testid="source-editor"></div>
