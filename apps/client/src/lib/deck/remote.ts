/** What two windows showing one deck tell each other. */
export type RemoteMessage = { type: 'step'; k: number } | { type: 'hello' };

/** The step this window shows, and what to do when another window moves. */
export interface RemoteHost {
	current(): number;
	onstep(k: number): void;
}

function remoteMessage(data: unknown): RemoteMessage | null {
	const raw = (typeof data === 'object' && data !== null ? data : {}) as { type?: unknown; k?: unknown };
	if (raw.type === 'hello') return { type: 'hello' };
	if (raw.type === 'step' && typeof raw.k === 'number' && Number.isFinite(raw.k)) return { type: 'step', k: raw.k };
	return null;
}

/**
 * Keeps the present window and the presenter view of deck `id` on the same step, through a
 * `BroadcastChannel`. A window that joins asks with `hello`; every other window answers its step.
 */
export function joinRemote(id: string, host: RemoteHost): { send(k: number): void; close(): void } {
	const channel = new BroadcastChannel(`diapo.present.${id}`);
	channel.onmessage = (event) => {
		const message = remoteMessage(event.data);
		if (message?.type === 'hello') channel.postMessage({ type: 'step', k: host.current() });
		if (message?.type === 'step') host.onstep(message.k);
	};
	channel.postMessage({ type: 'hello' });
	return {
		send: (k) => channel.postMessage({ type: 'step', k }),
		close: () => channel.close()
	};
}
