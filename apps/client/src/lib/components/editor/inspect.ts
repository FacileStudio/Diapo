import type { FullType, Shape, Style } from '@facile/planche';
import type { DeckEditor } from '$lib/editor/types';

/** The value every shape agrees on, or null when they differ or there is none. */
export function common<T>(shapes: Shape<FullType>[], read: (shape: Shape<FullType>) => T): T | null {
	if (shapes.length === 0) return null;
	const first = read(shapes[0]);
	return shapes.every((shape) => read(shape) === first) ? first : null;
}

/** Writes `fields` on every shape of the selection, one update per shape. */
export function apply(editor: DeckEditor, shapes: Shape<FullType>[], fields: Partial<Shape<FullType>>): void {
	for (const shape of shapes) editor.update(shape.id, fields);
}

/** Merges `patch` into each shape's own style, so a shape keeps the style fields the patch leaves out. */
export function restyle(editor: DeckEditor, shapes: Shape<FullType>[], patch: Style): void {
	for (const shape of shapes) editor.update(shape.id, { style: { ...shape.style, ...patch } });
}
