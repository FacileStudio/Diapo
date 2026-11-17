import type { EditorActions } from './actions';

/** Answers copy, cut and paste on the document while `node` itself has focus and `writable()` holds; answers the cleanup. */
export function listenClipboard(node: HTMLElement, actions: EditorActions, writable: () => boolean): () => void {
	const ours = () => document.activeElement === node && writable();
	const copy = (event: ClipboardEvent, cut: boolean) => {
		const text = ours() ? (cut ? actions.cut() : actions.copy()) : '';
		if (!text) return;
		event.clipboardData?.setData('text/plain', text);
		event.preventDefault();
	};
	const handlers = {
		copy: (event: ClipboardEvent) => copy(event, false),
		cut: (event: ClipboardEvent) => copy(event, true),
		paste: (event: ClipboardEvent) => {
			if (!ours()) return;
			event.preventDefault();
			actions.pasteText(event.clipboardData?.getData('text/plain') ?? '');
		}
	};
	const events = Object.entries(handlers) as [string, EventListener][];
	for (const [name, handler] of events) document.addEventListener(name, handler);
	return () => events.forEach(([name, handler]) => document.removeEventListener(name, handler));
}
