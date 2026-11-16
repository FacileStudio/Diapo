import { ApiError, backend } from '$lib/backend';
import type { ExportFormat, ExportJob } from '$lib/types';

/** Where one format stands in the export dialog. `soon` means the API cannot produce it yet. */
export type ExportState = 'idle' | 'working' | 'done' | 'failed' | 'soon';

/** One row of the export dialog. */
export interface ExportRow {
	state: ExportState;
	job: ExportJob | null;
}

/** The formats, in the order the dialog lists them. */
export const EXPORT_FORMATS: readonly ExportFormat[] = ['pptx', 'html', 'pdf', 'mp4'];

const unavailable = (error: unknown) => error instanceof ApiError && error.status === 503;

/**
 * The exports of one deck. When the dialog opens, `GET /api/exports/formats` says which formats
 * the API can produce now: the others show as coming soon. Nothing is started to find out. A job
 * that is not finished is polled every `interval` ms until it is `done` or `failed`.
 */
export class Exporter {
	rows = $state<Record<ExportFormat, ExportRow>>({
		pptx: { state: 'idle', job: null },
		html: { state: 'idle', job: null },
		pdf: { state: 'idle', job: null },
		mp4: { state: 'idle', job: null }
	});
	private stopped = false;

	constructor(
		private readonly id: string,
		private readonly interval = 1500
	) {}

	/** Asks the API which formats it can produce now. A failed answer leaves the rows as they are. */
	async probe(): Promise<void> {
		const { formats } = await backend.exportFormats().catch(() => ({ formats: [] }));
		for (const { format, available } of formats) {
			const row = this.rows[format];
			if (!available && row?.state === 'idle') row.state = 'soon';
			if (available && row?.state === 'soon') row.state = 'idle';
		}
	}

	/** Starts the export of `format` and follows its job to the end. */
	async start(format: ExportFormat): Promise<void> {
		if (this.rows[format].state === 'working') return;
		this.rows[format] = { state: 'working', job: null };
		try {
			await this.follow(format, await backend.startExport(this.id, format));
		} catch (error) {
			this.rows[format] = { state: unavailable(error) ? 'soon' : 'failed', job: null };
		}
	}

	/** Stops following jobs: the dialog's deck went away. */
	stop(): void {
		this.stopped = true;
	}

	private async follow(format: ExportFormat, first: ExportJob): Promise<void> {
		let job = first;
		this.rows[format] = { state: 'working', job };
		while (!this.stopped && (job.status === 'queued' || job.status === 'running')) {
			await new Promise((resolve) => setTimeout(resolve, this.interval));
			job = await backend.exportJob(job.id);
		}
		if (!this.stopped) this.rows[format] = { state: job.status === 'done' ? 'done' : 'failed', job };
	}
}
