import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import type { ExportFormat, ExportJob } from '$lib/types';
import { Exporter } from './exporting.svelte';

const job = (format: ExportFormat, status: ExportJob['status']): ExportJob => ({
	id: `job-${format}`,
	format,
	status,
	error: status === 'failed' ? 'boom' : '',
	file_name: `deck.${format}`,
	size: 10,
	download_url: status === 'done' ? `/api/exports/job-${format}/file` : null
});

const EXPORT_STATES = (exporter: Exporter) =>
	(['html', 'pptx', 'pdf', 'mp4'] as const).map((format) => exporter.rows[format].state);

const soon = () => Promise.reject(new ApiError(503, 'unavailable', 'not available yet'));

beforeEach(() => {
	vi.useFakeTimers();
	localStorage.clear();
});
afterEach(() => {
	vi.useRealTimers();
	vi.restoreAllMocks();
});

describe('Exporter', () => {
	it('finishes at once when the API answers a finished job', async () => {
		vi.spyOn(backend, 'startExport').mockResolvedValue(job('html', 'done'));
		const exporter = new Exporter('d1');
		await exporter.start('html');
		expect(exporter.rows.html).toMatchObject({ state: 'done', job: { download_url: '/api/exports/job-html/file' } });
	});

	it('polls a queued job until it is done', async () => {
		vi.spyOn(backend, 'startExport').mockResolvedValue(job('pptx', 'queued'));
		const read = vi.spyOn(backend, 'exportJob');
		read.mockResolvedValueOnce(job('pptx', 'running')).mockResolvedValueOnce(job('pptx', 'done'));
		const exporter = new Exporter('d1', 1000);
		const running = exporter.start('pptx');
		await vi.advanceTimersByTimeAsync(0);
		expect(exporter.rows.pptx.state).toBe('working');
		await vi.advanceTimersByTimeAsync(2000);
		await running;
		expect([exporter.rows.pptx.state, read.mock.calls.length]).toEqual(['done', 2]);
	});

	it('reports a job that fails, and a call that fails', async () => {
		const start = vi.spyOn(backend, 'startExport').mockResolvedValueOnce(job('html', 'failed'));
		const exporter = new Exporter('d1');
		await exporter.start('html');
		expect(exporter.rows.html).toMatchObject({ state: 'failed', job: { error: 'boom' } });
		start.mockRejectedValueOnce(new ApiError(0, 'network', 'offline'));
		await exporter.start('pptx');
		expect(exporter.rows.pptx.state).toBe('failed');
	});

	it('asks the API which formats work and starts nothing to find out', async () => {
		const start = vi.spyOn(backend, 'startExport');
		const formats = vi.spyOn(backend, 'exportFormats').mockResolvedValue({
			formats: [
				{ format: 'html', available: true },
				{ format: 'pptx', available: true, notes: 'raw scenes as placeholders' },
				{ format: 'pdf', available: false, reason: 'the capture service is not running' },
				{ format: 'mp4', available: false }
			]
		});
		const exporter = new Exporter('d1');
		await exporter.probe();
		expect(start).not.toHaveBeenCalled();
		expect(EXPORT_STATES(exporter)).toEqual(['idle', 'idle', 'soon', 'soon']);
		formats.mockResolvedValue({ formats: [{ format: 'pdf', available: true }] });
		await exporter.probe();
		expect(EXPORT_STATES(exporter)).toEqual(['idle', 'idle', 'idle', 'soon']);
	});

	it('keeps every format offered when the capability answer fails, and shows a 503 as coming soon', async () => {
		vi.spyOn(backend, 'exportFormats').mockRejectedValue(new ApiError(0, 'network', 'offline'));
		vi.spyOn(backend, 'startExport').mockImplementation(soon);
		const exporter = new Exporter('d1');
		await exporter.probe();
		expect(EXPORT_STATES(exporter)).toEqual(['idle', 'idle', 'idle', 'idle']);
		await exporter.start('pdf');
		expect(exporter.rows.pdf.state).toBe('soon');
	});
});
