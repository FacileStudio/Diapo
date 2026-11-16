import { backend } from '$lib/backend';

/** Where the share link of a deck stands. */
export type ShareStatus = 'loading' | 'off' | 'on' | 'failed';

/**
 * The share link of one deck. The API answers the link's URL once, when it creates it: `url`
 * holds it until the dialog closes, and nothing can read it again afterwards.
 */
export class Sharing {
	status = $state<ShareStatus>('loading');
	url = $state<string | null>(null);
	createdAt = $state<string | null>(null);
	busy = $state(false);

	constructor(private readonly id: string) {}

	async load(): Promise<void> {
		try {
			const link = await backend.share(this.id);
			this.status = link.shared ? 'on' : 'off';
			this.createdAt = link.created_at;
		} catch {
			this.status = 'failed';
		}
	}

	/** Shares the deck. With `rotate`, the live link dies and a new one replaces it. */
	async create(rotate = false): Promise<void> {
		await this.run(async () => {
			const link = await backend.createShare(this.id, rotate);
			this.status = 'on';
			this.createdAt = link.created_at;
			this.url = link.url ?? null;
		});
	}

	async revoke(): Promise<void> {
		await this.run(async () => {
			await backend.revokeShare(this.id);
			this.status = 'off';
			this.createdAt = null;
			this.url = null;
		});
	}

	/** Forgets the URL: the dialog closed, and the link is not shown again. */
	forget(): void {
		this.url = null;
	}

	private async run(action: () => Promise<void>): Promise<void> {
		if (this.busy) return;
		this.busy = true;
		try {
			await action();
		} finally {
			this.busy = false;
		}
	}
}

/** The token of a share URL `<origin>/p/<token>`, or null when `url` is not one. */
export function shareToken(url: string): string | null {
	try {
		return /^\/p\/([A-Za-z0-9_-]+)$/.exec(new URL(url).pathname)?.[1] ?? null;
	} catch {
		return null;
	}
}

/** The HTML that frames the shared deck at `url` in another site, at the stage ratio `width` by `height`. */
export function embedSnippet(url: string, title: string, width: number, height: number): string {
	const token = shareToken(url);
	if (!token) return '';
	const src = `${new URL(url).origin}/embed/${token}`;
	const safe = title.replace(/[<>"&]/g, '');
	const tall = Math.round((960 * height) / width);
	return `<iframe src="${src}" width="960" height="${tall}" title="${safe}" frameborder="0" allowfullscreen></iframe>`;
}
