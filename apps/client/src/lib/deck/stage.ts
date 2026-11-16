/** One scene of a compiled document, as its `scene:ready` message lists it. `first` is its first global step. */
export interface StageScene {
	start: number;
	duration: number;
	steps: number;
	first: number;
}

/** What a compiled document says once it is ready: its length, its last step and its scenes. */
export interface StageInfo {
	duration: number;
	steps: number;
	scenes: StageScene[];
}

/** What a compiled document says after every draw. `scene` is the index of the shown scene. */
export interface StageState {
	time: number;
	step: number;
	scene: number;
}

/** A message from a compiled document to its host, per `docs/format.md`. */
export type StageMessage = ({ type: 'scene:ready' } & StageInfo) | ({ type: 'scene:state' } & StageState);

const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);

function scenes(raw: unknown): StageScene[] | null {
	if (!Array.isArray(raw)) return null;
	const found = raw.map((row) => ({
		start: row?.start,
		duration: row?.duration,
		steps: row?.steps,
		first: row?.first
	}));
	return found.every((row) => Object.values(row).every(finite)) ? (found as StageScene[]) : null;
}

/** The scene message in `data`, or null when it is not one. A frame is untrusted: every field is checked. */
export function stageMessage(data: unknown): StageMessage | null {
	if (typeof data !== 'object' || data === null) return null;
	const raw = data as Record<string, unknown>;
	if (raw.type === 'scene:state' && finite(raw.time) && finite(raw.step) && finite(raw.scene)) {
		return { type: 'scene:state', time: raw.time, step: raw.step, scene: raw.scene };
	}
	const list = raw.type === 'scene:ready' ? scenes(raw.scenes) : null;
	if (!list || !finite(raw.duration) || !finite(raw.steps)) return null;
	return { type: 'scene:ready', duration: raw.duration, steps: raw.steps, scenes: list };
}

/** The global step that shows scene `index` with every step revealed; 0 when the scene is unknown. */
export function revealed(info: StageInfo | null, index: number): number {
	const scene = info?.scenes[Math.min(index, (info?.scenes.length ?? 1) - 1)];
	return scene ? scene.first + scene.steps : 0;
}

const MOVES: Record<string, number> = {
	ArrowRight: 1,
	ArrowDown: 1,
	PageDown: 1,
	' ': 1,
	Enter: 1,
	ArrowLeft: -1,
	ArrowUp: -1,
	PageUp: -1
};

/** The step a key leads to from `step` in a document whose last step is `last`, or null when the key moves nothing. */
export function keyStep(key: string, step: number, last: number): number | null {
	if (key === 'Home') return 0;
	if (key === 'End') return last;
	const move = MOVES[key];
	return move === undefined ? null : Math.min(Math.max(step + move, 0), last);
}
