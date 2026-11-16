import { backend, type SessionUser } from './backend';

/** The signed-in person, or nobody. */
class Session {
	user = $state<SessionUser | null>(null);
	ready = $state(false);

	async load(): Promise<void> {
		try {
			this.user = (await backend.session()).user;
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
