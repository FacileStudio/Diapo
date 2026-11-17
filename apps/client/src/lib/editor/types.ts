import type { Awareness } from 'y-protocols/awareness';
import type * as Y from 'yjs';
import type { BoardDoc, Box, FullType, Shape } from '@facile/planche';
import type { Connection, Stop } from '@facile/salon';

/** A slide as the rail and the stage read it: a top-level `frame` shape, in `order`. */
export interface Slide {
	id: string;
	name: string;
	order: string;
	shapes: Shape<FullType>[];
}

/** Someone else in the deck's room, with what they have selected. */
export interface Peer {
	client: number;
	name: string;
	color: string;
	slide: string | null;
	selection: string[];
}

/** The shapes a toolbar can insert. */
export type Insertable = 'text' | 'rect' | 'ellipse' | 'diamond' | 'image' | 'video';

/**
 * The live state of one open deck: the planche document in its salon room, the slides, the
 * current slide and selection, and every edit the UI makes. Every write is one Yjs transaction
 * with the planche `LOCAL` origin, so undo tracks only this client.
 */
export interface DeckEditor {
	readonly doc: BoardDoc;
	readonly awareness: Awareness;
	readonly size: { width: number; height: number };
	readonly connection: Connection;
	readonly stopped: { stop: Stop; reason: string } | null;
	readonly synced: boolean;
	readonly slides: Slide[];
	readonly current: string | null;
	readonly selection: string[];
	readonly peers: Peer[];
	readonly editing: string | null;
	readonly canUndo: boolean;
	readonly canRedo: boolean;

	show(slide: string): void;
	select(ids: string[]): void;
	edit(id: string | null): void;

	addSlide(after?: string): string;
	duplicateSlide(id: string): string;
	removeSlide(id: string): void;
	moveSlide(id: string, before: string | null): void;

	insert(kind: Insertable, at?: Partial<Box>, asset?: string): string;
	update(id: string, fields: Partial<Shape<FullType>>): void;
	remove(ids: string[]): void;
	reorder(id: string, where: 'front' | 'back' | 'forward' | 'backward'): void;

	text(id: string): Y.XmlFragment | undefined;
	notes(slide: string): Y.XmlFragment;

	undo(): void;
	redo(): void;
	destroy(): void;
}
