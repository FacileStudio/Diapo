import { describe, expect, it } from 'vitest';
import { keyStep, opening, revealed, sceneOfStep, stageMessage, type StageInfo } from './stage';

const INFO: StageInfo = {
	duration: 16,
	steps: 7,
	scenes: [
		{ start: 0, duration: 6, steps: 2, first: 0 },
		{ start: 6, duration: 8, steps: 3, first: 3 },
		{ start: 14, duration: 2, steps: 0, first: 7 }
	]
};

describe('stage', () => {
	it('reads the two messages of a compiled document and refuses anything else', () => {
		expect(stageMessage({ type: 'scene:ready', ...INFO })).toEqual({ type: 'scene:ready', ...INFO });
		expect(stageMessage({ type: 'scene:state', time: 1, step: 2, scene: 0, extra: 1 })).toEqual({
			type: 'scene:state',
			time: 1,
			step: 2,
			scene: 0
		});
		expect(stageMessage({ type: 'scene:state', time: 1, step: '2', scene: 0 })).toBeNull();
		expect(stageMessage({ type: 'scene:ready', duration: 1, steps: 0, scenes: [{ start: 0 }] })).toBeNull();
		expect(stageMessage({ type: 'bulle:ready' })).toBeNull();
		expect(stageMessage(null)).toBeNull();
	});

	it('maps global steps to scenes and back', () => {
		expect([0, 2, 3, 6, 7].map((step) => sceneOfStep(INFO, step))).toEqual([0, 0, 1, 1, 2]);
		expect([0, 1, 2, 9].map((index) => revealed(INFO, index))).toEqual([2, 6, 7, 7]);
		expect([opening(INFO, 1), opening(INFO, 5), revealed(null, 1)]).toEqual([3, 0, 0]);
	});

	it('moves one step per key and stays inside the deck', () => {
		expect(keyStep('ArrowRight', 2, 7)).toBe(3);
		expect(keyStep(' ', 7, 7)).toBe(7);
		expect(keyStep('PageUp', 0, 7)).toBe(0);
		expect(keyStep('ArrowLeft', 3, 7)).toBe(2);
		expect([keyStep('Home', 4, 7), keyStep('End', 4, 7)]).toEqual([0, 7]);
		expect(keyStep('a', 4, 7)).toBeNull();
	});
});
