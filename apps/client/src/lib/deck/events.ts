import { backend } from '$lib/backend';

/**
 * Follows the `changed` events of deck `id`; answers the function that stops following. The
 * first event of a connection only says where the deck is: `first` is true for it.
 */
export function watchDeck(id: string, onchanged: (rev: number, first: boolean) => void): () => void {
	let first = true;
	const source = new EventSource(backend.eventsUrl(id), { withCredentials: true });
	source.addEventListener('changed', (event) => {
		try {
			const rev = Number(JSON.parse((event as MessageEvent<string>).data).rev);
			if (Number.isFinite(rev)) onchanged(rev, first);
			first = false;
		} catch {
			return;
		}
	});
	return () => source.close();
}
