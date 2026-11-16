import { backend } from './backend';
import { DEFAULT_BULLE } from './bulle';
import type { SessionUser } from './types';

/** The signed-in person, or nobody, and the Bulle origin from `BULLE_URL`. */
class Session {
	user = $state<SessionUser | null>(null);
	bulle = $state(DEFAULT_BULLE);
	ready = $state(false);

	async load(): Promise<void> {
		try {
			const answer = await backend.session();
			this.user = answer.user;
			this.bulle = answer.bulle_url || DEFAULT_BULLE;
		} catch {
			this.user = null;
		}
		this.ready = true;
	}

	async logout(): Promise<void> {
		try {
			await backend.logout();
		} finally {
			this.user = null;
		}
	}
}

/** The one session of the app. */
export const session = new Session();
