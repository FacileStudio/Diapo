import { rotatedBox, type Box, type FullType, type Handle, type Point, type Shape } from '@facile/planche';
import { stageCommand, type StageCommand } from './keys';
import { areaOf, marquee, moveDrag, resizeDrag, rotateDrag, type Guides } from './pointer';
import { duplicateShapes } from './shapes';
import type { DeckEditor } from './types';
import type { Fields } from './write';

/** What a pointer went down on. */
export type Target =
	| { kind: 'shape'; id: string }
	| { kind: 'handle'; handle: Handle }
	| { kind: 'rotate' }
	| { kind: 'empty' };

type Drag =
	| { kind: 'move' | 'resize' | 'rotate'; start: Point; shapes: Shape<FullType>[]; handle?: Handle }
	| { kind: 'marquee'; start: Point; base: string[] };

type Modifiers = Pick<PointerEvent, 'shiftKey' | 'altKey'>;

const NO_GUIDES: Guides = { x: [], y: [] };

/** The pointer and keyboard behaviour of the stage over one deck editor, with the drag previewed locally. */
export class StageController {
	readonly editor: DeckEditor;
	scale = $state(1);
	preview = $state.raw<Map<string, Fields>>(new Map());
	guides = $state.raw<Guides>(NO_GUIDES);
	area = $state.raw<Box | null>(null);
	#drag: Drag | null = null;

	constructor(editor: DeckEditor) {
		this.editor = editor;
	}

	/** The shapes of the shown slide, in paint order. */
	get shapes(): Shape<FullType>[] {
		return this.editor.slides.find((slide) => slide.id === this.editor.current)?.shapes ?? [];
	}

	/** Whether a pointer drag is under way. */
	get dragging(): boolean {
		return this.#drag !== null;
	}

	/** `shape` with the drag preview applied. */
	view(shape: Shape<FullType>): Shape<FullType> {
		const fields = this.preview.get(shape.id);
		return fields ? ({ ...shape, ...fields } as Shape<FullType>) : shape;
	}

	#selected(): Shape<FullType>[] {
		const ids = new Set(this.editor.selection);
		return this.shapes.filter((shape) => ids.has(shape.id));
	}

	down(target: Target, at: Point, event: Modifiers): void {
		const { editor } = this;
		if (target.kind === 'empty') {
			if (editor.editing) editor.edit(null);
			const base = event.shiftKey ? editor.selection : [];
			if (!event.shiftKey) editor.select([]);
			this.#drag = { kind: 'marquee', start: at, base };
			return;
		}
		if (target.kind === 'shape') return this.#press(target.id, at, event);
		const kind = target.kind === 'rotate' ? 'rotate' : 'resize';
		const handle = target.kind === 'handle' ? target.handle : undefined;
		this.#drag = { kind, start: at, shapes: this.#selected(), handle };
	}

	#press(id: string, at: Point, event: Modifiers): void {
		const { editor } = this;
		if (editor.editing === id) return;
		if (event.shiftKey) {
			const has = editor.selection.includes(id);
			editor.select(has ? editor.selection.filter((item) => item !== id) : [...editor.selection, id]);
			return;
		}
		if (!editor.selection.includes(id)) editor.select([id]);
		this.#drag = { kind: 'move', start: at, shapes: this.#selected() };
	}

	move(at: Point, event: Modifiers): void {
		const drag = this.#drag;
		if (!drag) return;
		const delta = { x: at.x - drag.start.x, y: at.y - drag.start.y };
		if (drag.kind === 'marquee') {
			this.area = areaOf(drag.start, at);
			this.editor.select([...drag.base, ...marquee(this.shapes, this.area)]);
		} else if (drag.kind === 'move') {
			const moved = moveDrag(drag.shapes, delta, event.altKey ? [] : this.#targets(drag.shapes), 6 / this.scale);
			this.preview = moved.changes;
			this.guides = moved.guides;
		} else if (drag.kind === 'resize' && drag.handle) {
			this.preview = resizeDrag(drag.shapes, drag.handle, delta, event.shiftKey);
		} else if (drag.kind === 'rotate' && drag.shapes.length === 1 && !drag.shapes[0].locked) {
			const [shape] = drag.shapes;
			this.preview = new Map([[shape.id, { rot: rotateDrag(shape, drag.start, at, event.shiftKey) }]]);
		}
	}

	#targets(moving: Shape<FullType>[]): Box[] {
		const skip = new Set(moving.map((shape) => shape.id));
		const { width, height } = this.editor.size;
		const others = this.shapes.filter((shape) => !skip.has(shape.id)).map((shape) => rotatedBox(shape));
		return [{ x: 0, y: 0, w: width, h: height }, ...others];
	}

	up(): void {
		const changes = this.preview;
		this.#drag = null;
		this.preview = new Map();
		this.guides = NO_GUIDES;
		this.area = null;
		if (changes.size) this.editor.doc.transact(() => changes.forEach((fields, id) => this.editor.update(id, fields)));
	}

	/** Opens the text of shape `id` for typing. */
	open(id: string): void {
		if (this.editor.text(id)) this.editor.edit(id);
	}

	/** Runs the command of a key press; answers whether the stage used it. */
	key(event: KeyboardEvent): boolean {
		const command = stageCommand(event, this.editor.editing !== null);
		if (!command) return false;
		this.#run(command);
		return true;
	}

	#run(command: StageCommand): void {
		const { editor } = this;
		const selected = this.#selected();
		if (command.kind === 'nudge') {
			const free = selected.filter((shape) => !shape.locked);
			const nudge = (shape: Shape<FullType>) => ({ x: shape.x + command.dx, y: shape.y + command.dy });
			editor.doc.transact(() => free.forEach((shape) => editor.update(shape.id, nudge(shape))));
		} else if (command.kind === 'remove') editor.remove(editor.selection);
		else if (command.kind === 'undo') editor.undo();
		else if (command.kind === 'redo') editor.redo();
		else if (command.kind === 'duplicate') editor.select(duplicateShapes(editor.doc, editor.selection));
		else if (command.kind === 'edit' && selected.length === 1) this.open(selected[0].id);
		else if (command.kind === 'leave' && editor.editing) editor.edit(null);
		else if (command.kind === 'leave') editor.select([]);
		else if (command.kind === 'all') editor.select(this.shapes.map((shape) => shape.id));
	}
}
