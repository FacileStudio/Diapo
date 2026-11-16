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

	it('shows a format the API answers 503 for as coming soon, without an error', async () => {
		const start = vi.spyOn(backend, 'startExport').mockImplementation(soon);
		const exporter = new Exporter('d1');
		await exporter.probe();
		expect(start.mock.calls.map(([, format]) => format).sort()).toEqual(['mp4', 'pdf']);
		expect([exporter.rows.pdf.state, exporter.rows.mp4.state, exporter.rows.html.state]).toEqual([
			'soon',
			'soon',
			'idle'
		]);
	});

	it('lights a format up by itself the day the API produces it, and stops probing it', async () => {
		const start = vi.spyOn(backend, 'startExport');
		start.mockImplementation((_id, format) => (format === 'pdf' ? Promise.resolve(job('pdf', 'done')) : soon()));
		const first = new Exporter('d1');
		await first.probe();
		expect([first.rows.pdf.state, first.rows.mp4.state]).toEqual(['done', 'soon']);
		start.mockClear();
		const later = new Exporter('d1');
		await later.probe();
		expect(start.mock.calls.map(([, format]) => format)).toEqual(['mp4']);
		expect(later.rows.pdf.state).toBe('idle');
	});
});
