import { ApiError, backend } from '$lib/backend';
import type { ExportFormat, ExportJob } from '$lib/types';
import { ASK_AGAIN_AFTER, asked, known, learn, refused } from './export-memory';

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
 * The exports of one deck. A format this browser has never seen work is asked when the dialog
 * opens: `503 unavailable` shows it as coming soon and is believed for `ASK_AGAIN_AFTER`, and an
 * answer with a job shows that job, so a format lights up by itself once the API can produce it. A job that is not
 * finished is polled every `interval` ms until it is `done` or `failed`.
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

	/** Asks the API about every format this browser has not seen work yet. */
	async probe(): Promise<void> {
		const unsure = EXPORT_FORMATS.filter((format) => !known().includes(format) && this.rows[format].state === 'idle');
		for (const format of unsure) this.rows[format].state = 'soon';
		const due = unsure.filter((format) => Date.now() - (asked()[format] ?? 0) >= ASK_AGAIN_AFTER);
		await Promise.all(due.map((format) => this.start(format)));
	}

	/** Starts the export of `format` and follows its job to the end. */
	async start(format: ExportFormat): Promise<void> {
		const before = this.rows[format].state;
		if (before === 'working') return;
		if (before !== 'soon') this.rows[format] = { state: 'working', job: null };
		try {
			const job = await backend.startExport(this.id, format);
			learn(format);
			await this.follow(format, job);
		} catch (error) {
			if (unavailable(error)) refused(format);
			this.rows[format] = { state: unavailable(error) ? 'soon' : before === 'soon' ? 'soon' : 'failed', job: null };
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
