import { revealed, type StageInfo, type StageState } from './stage';

/**
 * Where a compiled document stands and where its host wants it. `want` is the global step the
 * frame is asked to draw; `at` is what it last drew. While following, a document that reloads
 * goes back to the focused scene, fully revealed.
 */
export class Playhead {
	info = $state<StageInfo | null>(null);
	at = $state<StageState>({ time: 0, step: 0, scene: 0 });
	want = $state(0);
	private scene = 0;

	constructor(private follow = true) {}

	ready(info: StageInfo): void {
		this.info = info;
		this.want = this.follow ? revealed(info, this.scene) : Math.min(this.want, info.steps);
	}

	drawn(state: StageState): void {
		this.at = state;
	}

	/** Shows scene `index` with every step revealed, and keeps showing it across reloads. */
	focus(index: number): void {
		this.scene = index;
		this.follow = true;
		this.want = revealed(this.info, index);
	}

	/** Goes to the global step `k`, clamped to the document once it is known. */
	go(k: number): void {
		this.follow = false;
		this.want = Math.min(Math.max(Math.round(k), 0), this.info?.steps ?? Math.round(k));
	}

	move(delta: number): void {
		this.go(this.want + delta);
	}

	/** The shown scene and step, counted from 1 for people: scene 2 of 5, step 1 of 3. */
	get position(): { scene: number; scenes: number; step: number; steps: number } {
		const current = this.info?.scenes[this.at.scene];
		return {
			scene: this.at.scene + 1,
			scenes: this.info?.scenes.length ?? 0,
			step: current ? this.at.step - current.first : 0,
			steps: current?.steps ?? 0
		};
	}
}
