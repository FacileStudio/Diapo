import type { FullType, Point, Shape, Style } from '@facile/planche';
import { alignShapes, distributeShapes, type Axis, type Edge } from './arrange';
import { decodeShapes, encodeShapes, pasteShapes, textClip, type Clip } from './clipboard';
import { duplicateShapes, type Reorder } from './shapes';
import type { DeckEditor, Slide } from './types';
import type { Fields } from './write';

/** A text box's horizontal alignment, `justify` included. */
export type HAlign = 'left' | 'center' | 'right' | 'justify';

/** A text box's vertical alignment. */
export type VAlign = 'top' | 'middle' | 'bottom';

/** The text alignment a shape's style carries under the `full` profile. */
export type TextAlign = { align?: HAlign; valign?: VAlign };

const ORDER: Record<Reorder, 1 | -1> = { front: 1, backward: 1, forward: -1, back: -1 };

let memory = '';

/** What the menus, the toolbar and the keyboard do to the selection of one deck editor. */
export class EditorActions {
	readonly editor: DeckEditor;

	constructor(editor: DeckEditor) {
		this.editor = editor;
	}

	/** The slide on show. */
	get slide(): Slide | undefined {
		return this.editor.slides.find((slide) => slide.id === this.editor.current);
	}

	/** The selected shapes of the slide on show, in paint order. */
	get selected(): Shape<FullType>[] {
		const ids = new Set(this.editor.selection);
		return (this.slide?.shapes ?? []).filter((shape) => ids.has(shape.id));
	}

	/** Copies the selection to this tab and to the system clipboard; answers the clipboard text, empty for none. */
	copy(): string {
		if (this.selected.length === 0) return '';
		memory = encodeShapes(this.editor.doc, this.selected.map((shape) => shape.id));
		void navigator.clipboard?.writeText(memory).catch(() => undefined);
		return memory;
	}

	/** Copies the selection, then deletes it. */
	cut(): string {
		const text = this.copy();
		if (text) this.editor.remove(this.selected.map((shape) => shape.id));
		return text;
	}

	/** Pastes clipboard `text` on the slide on show: Diapo shapes as they were, other text as a text box. */
	pasteText(text: string, at?: Point): string[] {
		const slide = this.slide;
		const clip: Clip | null = decodeShapes(text) ?? (text.trim() ? textClip(text.trim(), this.editor.size) : null);
		if (!slide || !clip || clip.shapes.length === 0) return [];
		const ids = pasteShapes(this.editor.doc, slide, clip, at);
		this.editor.select(ids);
		return ids;
	}

	/** Pastes what the system clipboard holds, or what this tab copied last when it cannot be read. */
	async paste(at?: Point): Promise<string[]> {
		const read = await navigator.clipboard?.readText().catch(() => '');
		return this.pasteText(read || memory, at);
	}

	/** Whether this tab copied shapes, so a paste has something even without clipboard access. */
	get copied(): boolean {
		return memory !== '';
	}

	/** Copies the selection on top, offset, and selects the copies. */
	duplicate(): void {
		this.editor.select(duplicateShapes(this.editor.doc, this.selected.map((shape) => shape.id)));
	}

	/** Moves every selected shape in the paint order, keeping their order among themselves. */
	reorder(where: Reorder): void {
		const shapes = ORDER[where] === 1 ? this.selected : [...this.selected].reverse();
		this.editor.doc.transact(() => shapes.forEach((shape) => this.editor.reorder(shape.id, where)));
	}

	/** Lines the selection up on `edge`; a lone shape lines up on the slide. */
	align(edge: Edge): void {
		const { width, height } = this.editor.size;
		this.#write(alignShapes(this.selected, edge, { x: 0, y: 0, w: width, h: height }));
	}

	/** Spreads three or more selected shapes evenly along `axis`. */
	distribute(axis: Axis): void {
		this.#write(distributeShapes(this.selected, axis));
	}

	/** Locks or unlocks every selected shape. */
	lock(locked: boolean): void {
		this.#each(() => ({ locked: locked || undefined }));
	}

	/** Moves every selected shape's reveal step by `delta`, never below 0 (shown with the slide). */
	step(delta: number): void {
		this.#each((shape) => ({ step: Math.max(0, (shape.step ?? 0) + delta) || undefined }));
	}

	/** Sets the text alignment of every selected shape, keeping the rest of its style. */
	textAlign(patch: TextAlign): void {
		this.#each((shape) => ({ style: { ...shape.style, ...patch } as Style }));
	}

	#each(fields: (shape: Shape<FullType>) => Fields): void {
		const shapes = this.selected;
		this.editor.doc.transact(() => shapes.forEach((shape) => this.editor.update(shape.id, fields(shape))));
	}

	#write(changes: Map<string, Fields>): void {
		this.editor.doc.transact(() => changes.forEach((fields, id) => this.editor.update(id, fields)));
	}
}
