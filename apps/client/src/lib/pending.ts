import { ApiError } from './backend';
import type { AppKey } from './i18n/messages';

/** How the confirm page describes an action's state: ready to run, or its deck changed or vanished since. */
export const STATES: Record<string, { tone: 'success' | 'warning' | 'danger'; label: AppKey }> = {
	ready: { tone: 'success', label: 'pending.ready' },
	changed: { tone: 'warning', label: 'pending.changed' },
	missing: { tone: 'danger', label: 'pending.gone' }
};

/** Whether `error` is the confirm page's 403: the session is not the person's own browser sign-in. */
export function needsBrowserLogin(error: unknown): boolean {
	return error instanceof ApiError && error.status === 403 && error.code === 'permission_denied';
}
