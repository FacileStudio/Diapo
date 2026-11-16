import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '$lib/backend';
import { Saver } from './saver.svelte';

function fake() {
	return {
		put: vi.fn(async (_source: string, rev: number) => ({ rev: rev + 1 })),
		get: vi.fn(async () => ({ source: 'theirs', rev: 9 })),
		onreplace: vi.fn((_source: string) => undefined)
	};
}

function opened(io: ReturnType<typeof fake>): Saver {
	const saver = new Saver(io, 500);
	saver.open({ source: 'a', rev: 1 });
	return saver;
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('Saver', () => {
	it('saves once after the last edit, with the revision it read', async () => {
		const io = fake();
		const saver = opened(io);
		saver.edit('ab');
		await vi.advanceTimersByTimeAsync(300);
		saver.edit('abc');
		expect(saver.status).toBe('dirty');
		await vi.advanceTimersByTimeAsync(500);
		expect(io.put.mock.calls).toEqual([['abc', 1]]);
		expect([saver.status, saver.rev]).toEqual(['clean', 2]);
	});

	it('saves again when the buffer moved during a save', async () => {
		const io = fake();
		const saver = opened(io);
		saver.edit('ab');
		const first = saver.flush();
		saver.edit('abc');
		expect(saver.status).toBe('saving');
		await first;
		expect(saver.status).toBe('dirty');
		await vi.advanceTimersByTimeAsync(500);
		expect(io.put.mock.calls).toEqual([
			['ab', 1],
			['abc', 2]
		]);
		expect(saver.status).toBe('clean');
	});

	it('stops on a stale revision, then overwrites with the fresh one', async () => {
		const io = fake();
		io.put.mockRejectedValueOnce(new ApiError(409, 'already_exists', 'stale'));
		const saver = opened(io);
		saver.edit('mine');
		await saver.flush();
		expect(saver.status).toBe('conflict');
		saver.edit('mine too');
		await vi.advanceTimersByTimeAsync(2000);
		expect(io.put).toHaveBeenCalledTimes(1);
		await saver.overwrite();
		expect(io.put.mock.calls[1]).toEqual(['mine too', 9]);
		expect([saver.status, saver.rev]).toEqual(['clean', 10]);
	});

	it('reloads theirs out of a conflict', async () => {
		const io = fake();
		io.put.mockRejectedValueOnce(new ApiError(409, 'already_exists', 'stale'));
		const saver = opened(io);
		saver.edit('mine');
		await saver.flush();
		await saver.reload();
		expect(io.onreplace).toHaveBeenCalledWith('theirs');
		expect([saver.status, saver.rev, saver.buffer]).toEqual(['clean', 9, 'theirs']);
	});

	it('keeps the scene errors of a refused source until the next save passes', async () => {
		const io = fake();
		const errors = [{ line: 4, message: 'step 2 is past `steps: 1`' }];
		io.put.mockRejectedValueOnce(new ApiError(422, 'invalid_argument', 'invalid', errors));
		const saver = opened(io);
		saver.edit('bad');
		await saver.flush();
		expect([saver.status, saver.errors, saver.rev]).toEqual(['invalid', errors, 1]);
		saver.edit('good');
		await vi.advanceTimersByTimeAsync(500);
		expect([saver.status, saver.errors]).toEqual(['clean', []]);
	});

	it('marks a failed save and retries on the next edit', async () => {
		const io = fake();
		io.put.mockRejectedValueOnce(new ApiError(0, 'network', 'offline'));
		const saver = opened(io);
		saver.edit('x');
		await saver.flush();
		expect(saver.status).toBe('failed');
		saver.edit('xy');
		await vi.advanceTimersByTimeAsync(500);
		expect(saver.status).toBe('clean');
	});

	it('ignores its own revision in a changed event', async () => {
		const io = fake();
		const saver = opened(io);
		saver.edit('ab');
		const saving = saver.flush();
		await saver.remote(2);
		await saving;
		await saver.remote(2);
		expect(io.get).not.toHaveBeenCalled();
		expect(saver.status).toBe('clean');
	});

	it('reloads on an outside write when the buffer is clean', async () => {
		const io = fake();
		const saver = opened(io);
		await saver.remote(9);
		expect(io.onreplace).toHaveBeenCalledWith('theirs');
		expect([saver.status, saver.rev]).toEqual(['clean', 9]);
	});

	it('raises a conflict on an outside write over unsaved text, also one seen during a save', async () => {
		const io = fake();
		const dirty = opened(io);
		dirty.edit('mine');
		await dirty.remote(5);
		expect(dirty.status).toBe('conflict');
		expect(io.get).not.toHaveBeenCalled();

		const saving = opened(io);
		saving.edit('mine');
		const pending = saving.flush();
		saving.edit('mine, more');
		await saving.remote(7);
		await pending;
		expect(saving.status).toBe('conflict');
	});

	it('reloads after a save when an outside write landed during it and nothing else was typed', async () => {
		const io = fake();
		const saver = opened(io);
		saver.edit('mine');
		const pending = saver.flush();
		await saver.remote(7);
		await pending;
		await vi.advanceTimersByTimeAsync(0);
		expect([saver.status, saver.rev, saver.buffer]).toEqual(['clean', 9, 'theirs']);
	});

	it('tells a rename or an asset event from the echo of its own save', async () => {
		const io = fake();
		const saver = opened(io);
		expect(saver.touched(1)).toBe(true);
		saver.edit('ab');
		await saver.flush();
		expect([saver.touched(2), saver.touched(2), saver.touched(3)]).toEqual([false, true, false]);
		saver.edit('abc');
		const saving = saver.flush();
		await saver.remote(3);
		await saving;
		expect(saver.touched(3)).toBe(true);
	});
});
