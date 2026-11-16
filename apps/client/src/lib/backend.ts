import type {
	Asset,
	AssistantProfile,
	Deck,
	DeckSummary,
	ExportCapability,
	ExportFormat,
	ExportJob,
	PendingAction,
	PublicDeck,
	SceneError,
	SessionUser,
	ShareLink
} from './types';

/** The suite error codes, from tronc's `statusByCode`. */
export type ErrorCode =
	| 'invalid_argument'
	| 'unauthenticated'
	| 'permission_denied'
	| 'not_found'
	| 'already_exists'
	| 'failed_precondition'
	| 'resource_exhausted'
	| 'rate_limited'
	| 'method_not_allowed'
	| 'unavailable'
	| 'internal'
	| 'network';

const CODES: Record<number, ErrorCode> = {
	400: 'invalid_argument',
	401: 'unauthenticated',
	403: 'permission_denied',
	404: 'not_found',
	405: 'method_not_allowed',
	409: 'already_exists',
	422: 'invalid_argument',
	412: 'failed_precondition',
	413: 'resource_exhausted',
	429: 'rate_limited',
	503: 'unavailable'
};

const SAFE = new Set(['GET', 'HEAD', 'OPTIONS']);

/** The API base. Same origin: the Vite proxy in development, the Rust binary in production. */
export const API = '/api';

/** A failed API call, carrying the suite envelope `{"error":{"code","message"}}`. */
export class ApiError extends Error {
	constructor(
		readonly status: number,
		readonly code: ErrorCode,
		message: string,
		readonly errors: SceneError[] = []
	) {
		super(message);
		this.name = 'ApiError';
	}
}

type Envelope = { error?: { code?: string; message?: string; errors?: unknown }; errors?: unknown };

function sceneErrors(body: Envelope): SceneError[] {
	const found = body.errors ?? body.error?.errors;
	if (!Array.isArray(found)) return [];
	return found
		.filter((row): row is SceneError => typeof row?.line === 'number' && typeof row?.message === 'string')
		.map((row) => ({ line: row.line, message: row.message }));
}

/** Reads the suite error envelope of a failed response into an `ApiError`, with the scene errors of a 422. */
export async function toApiError(response: Response): Promise<ApiError> {
	const fallback = CODES[response.status] ?? 'internal';
	try {
		const body = (await response.json()) as Envelope;
		const code = (body.error?.code as ErrorCode | undefined) ?? fallback;
		return new ApiError(response.status, code, body.error?.message ?? response.statusText, sceneErrors(body));
	} catch {
		return new ApiError(response.status, fallback, response.statusText || `HTTP ${response.status}`);
	}
}

/** Calls the API: cookies included, the CSRF header on every mutation, the error envelope thrown typed. */
export async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const method = (init.method ?? 'GET').toUpperCase();
	const headers = new Headers(init.headers);
	if (typeof init.body === 'string' && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
	if (!SAFE.has(method)) headers.set('X-Facile-CSRF', '1');
	let response: Response;
	try {
		response = await fetch(`${API}${path}`, {
			credentials: 'include',
			...init,
			method,
			headers
		});
	} catch {
		throw new ApiError(0, 'network', 'Network error');
	}
	if (!response.ok) throw await toApiError(response);
	if (response.status === 204) return undefined as T;
	return (await response.json()) as T;
}

const send = (method: string, body?: unknown): RequestInit => ({
	method,
	body: body === undefined ? undefined : JSON.stringify(body)
});

const deck = (id: string, rest = '') => `/decks/${encodeURIComponent(id)}${rest}`;

const action = (id: string, verb: string) => `/mcp/pending/${encodeURIComponent(id)}/${verb}`;

function upload(id: string, file: File): Promise<Asset> {
	const form = new FormData();
	form.append('file', file);
	return request<Asset>(deck(id, '/assets'), { method: 'POST', body: form });
}

/** Which compiled document a frame shows: the whole deck, or one scene (from 0). A new `stamp` loads it again. */
export type FrameQuery = { scene?: number; stamp?: string | number };

function frameUrl(id: string, query: FrameQuery = {}): string {
	const params = new URLSearchParams();
	if (query.scene !== undefined) params.set('scene', String(query.scene));
	if (query.stamp !== undefined) params.set('v', String(query.stamp));
	const text = params.toString();
	return `${API}${deck(id, '/frame')}${text ? `?${text}` : ''}`;
}

/** Every call the client makes, one method per route. */
export const backend = {
	session: () => request<{ user: SessionUser | null; bulle_url?: string }>('/auth/session'),
	loginUrl: () => `${API}/auth/oidc`,
	logout: () => request<unknown>('/auth/logout', send('POST')),
	decks: () => request<DeckSummary[]>('/decks'),
	trashed: () => request<DeckSummary[]>('/decks?trashed=true'),
	create: (title: string, source?: string) => request<Deck>('/decks', send('POST', { title, source })),
	deck: (id: string) => request<Deck>(deck(id)),
	save: (id: string, source: string, rev: number) =>
		request<{ rev: number }>(deck(id, '/source'), send('PUT', { source, rev })),
	rename: (id: string, title: string) => request<Deck>(deck(id), send('PATCH', { title })),
	trash: (id: string, trashed: boolean) => request<Deck>(deck(id), send('PATCH', { trashed })),
	destroy: (id: string) => request<void>(deck(id), send('DELETE')),
	eventsUrl: (id: string) => `${API}${deck(id, '/events')}`,
	frameUrl,
	assets: (id: string) => request<{ assets: Asset[] }>(deck(id, '/assets')).then((found) => found.assets),
	upload,
	removeAsset: (id: string, name: string) => request<void>(deck(id, `/assets/${encodeURI(name)}`), send('DELETE')),
	share: (id: string) => request<ShareLink>(deck(id, '/share')),
	createShare: (id: string, rotate = false) =>
		request<ShareLink>(deck(id, `/share${rotate ? '?rotate=true' : ''}`), send('POST')),
	revokeShare: (id: string) => request<void>(deck(id, '/share'), send('DELETE')),
	publicDeck: (token: string) =>
		request<PublicDeck>(`/public/decks/${encodeURIComponent(token)}`, { credentials: 'omit' }),
	startExport: (id: string, format: ExportFormat) => request<ExportJob>(deck(id, '/exports'), send('POST', { format })),
	exportFormats: () => request<{ formats: ExportCapability[] }>('/exports/formats'),
	exportJob: (id: string) => request<ExportJob>(`/exports/${encodeURIComponent(id)}`),
	assistant: () => request<AssistantProfile>('/assistant'),
	actions: () => request<{ actions: PendingAction[]; truncated?: boolean }>('/mcp/pending'),
	confirmAction: (id: string) => request<unknown>(action(id, 'confirm'), send('POST')),
	dismissAction: (id: string) => request<unknown>(action(id, 'dismiss'), send('POST'))
};
