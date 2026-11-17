import * as Y from 'yjs';
import { Awareness } from 'y-protocols/awareness';
import type { WebsocketProvider } from 'y-websocket';
import { BoardDoc, createUndo, richText, type Box, type FullType, type Shape } from '@facile/planche';
import { publish, readPeers, type DeckUser } from './presence';
import { readSize, readSlides } from './read';
import { openDeckRoom, type Connection, type Room, type Stop } from './room';
import { insertShape, reorderShape, type Reorder } from './shapes';
import { addSlide, duplicateSlide, moveSlide, removeSlides } from './slides';
import type { DeckEditor, Insertable, Peer, Slide } from './types';
import { emptyRich, removeShapes, updateShape } from './write';

/** The live editor of one deck over a planche `full` document; see `DeckEditor`. */
export class DeckEditorState implements DeckEditor {
	readonly doc: BoardDoc;
	readonly awareness: Awareness;
	readonly provider: WebsocketProvider | null;
	size = $state({ width: 1920, height: 1080 });
	connection = $state<Connection>('connecting');
	stopped = $state<{ stop: Stop; reason: string } | null>(null);
	synced = $state(false);
	slides = $state.raw<Slide[]>([]);
	selection = $state.raw<string[]>([]);
	peers = $state.raw<Peer[]>([]);
	editing = $state<string | null>(null);
	canUndo = $state(false);
	canRedo = $state(false);
	#shown = $state<string | null>(null);
	#user: DeckUser;
	#room: Room | null;
	#undo: Y.UndoManager;
	#cleanup: (() => void)[] = [];

	constructor(doc: BoardDoc, awareness: Awareness, user: DeckUser, room: Room | null = null) {
		this.doc = doc;
		this.awareness = awareness;
		this.provider = room?.provider ?? null;
		this.#user = user;
		this.#room = room;
		this.#undo = createUndo(doc);
		this.#watch();
		this.#refresh();
		this.#share();
	}

	get current(): string | null {
		const shown = this.#shown;
		if (shown && this.slides.some((slide) => slide.id === shown)) return shown;
		return this.slides[0]?.id ?? null;
	}

	#watch(): void {
		const refresh = () => this.#refresh();
		const peers = () => (this.peers = readPeers(this.awareness));
		const history = () => {
			this.canUndo = this.#undo.canUndo();
			this.canRedo = this.#undo.canRedo();
		};
		this.doc.shapes.observeDeep(refresh);
		this.doc.meta.observe(refresh);
		this.awareness.on('change', peers);
		for (const event of ['stack-item-added', 'stack-item-popped', 'stack-cleared'] as const)
			this.#undo.on(event, history);
		this.#cleanup.push(
			() => this.doc.shapes.unobserveDeep(refresh),
			() => this.doc.meta.unobserve(refresh),
			() => this.awareness.off('change', peers)
		);
	}

	#refresh(): void {
		this.size = readSize(this.doc);
		this.slides = readSlides(this.doc);
		const known = new Set(this.slides.flatMap((slide) => slide.shapes.map((shape) => shape.id)));
		const kept = this.selection.filter((id) => known.has(id));
		if (kept.length !== this.selection.length) this.select(kept);
		if (this.editing && !known.has(this.editing)) this.editing = null;
	}

	#share(): void {
		publish(this.awareness, this.#user, this.current, this.selection);
	}

	/** Takes the room's connection state. */
	onconnection(state: Connection): void {
		this.connection = state;
	}

	/** Takes a terminal close of the room: the editor stays readable, nothing more syncs. */
	onstop(stop: Stop, reason: string): void {
		this.stopped = { stop, reason };
		this.connection = 'stopped';
	}

	/** Takes the room's first sync. */
	onsynced(): void {
		this.synced = true;
		this.#refresh();
	}

	show(slide: string): void {
		if (slide === this.current) return;
		this.#shown = slide;
		this.editing = null;
		this.selection = [];
		this.#share();
	}

	select(ids: string[]): void {
		this.selection = [...new Set(ids)];
		if (this.editing && !this.selection.includes(this.editing)) this.editing = null;
		this.#share();
	}

	edit(id: string | null): void {
		this.editing = id && this.text(id) ? id : null;
		if (this.editing) this.select([this.editing]);
	}

	addSlide(after?: string): string {
		const id = addSlide(this.doc, after ?? this.current ?? undefined);
		this.show(id);
		return id;
	}

	duplicateSlide(id: string): string {
		const copy = duplicateSlide(this.doc, id);
		if (copy) this.show(copy);
		return copy;
	}

	removeSlide(id: string): void {
		const index = this.slides.findIndex((slide) => slide.id === id);
		removeSlides(this.doc, [id]);
		const next = this.slides[Math.min(index, this.slides.length - 1)];
		if (next && id === this.#shown) this.show(next.id);
	}

	moveSlide(id: string, before: string | null): void {
		moveSlide(this.doc, id, before);
	}

	insert(kind: Insertable, at?: Partial<Box>, asset?: string): string {
		const shown = this.current ?? this.addSlide();
		const target = this.slides.find((item) => item.id === shown);
		if (!target) return '';
		const id = insertShape(this.doc, target, kind, at, asset);
		this.select([id]);
		if (kind === 'text') this.edit(id);
		return id;
	}

	update(id: string, fields: Partial<Shape<FullType>>): void {
		updateShape(this.doc, id, fields);
	}

	remove(ids: string[]): void {
		const frames = new Set(this.slides.map((slide) => slide.id));
		const slides = ids.filter((id) => frames.has(id));
		if (slides.length) removeSlides(this.doc, slides);
		removeShapes(this.doc, ids.filter((id) => !frames.has(id)));
	}

	reorder(id: string, where: Reorder): void {
		reorderShape(this.doc, id, where);
	}

	text(id: string): Y.XmlFragment | undefined {
		const map = this.doc.shapes.get(id);
		return map ? richText(map, 'text') : undefined;
	}

	notes(slide: string): Y.XmlFragment {
		const map = this.doc.shapes.get(slide);
		const found = map && richText(map, 'notes');
		if (found || !map) return found ?? emptyRich();
		const fragment = emptyRich();
		const before = this.#undo.undoStack.length;
		this.#undo.stopCapturing();
		this.doc.transact(() => map.set('notes', fragment));
		if (this.#undo.undoStack.length > before) this.#undo.undoStack.pop();
		this.#undo.stopCapturing();
		this.canUndo = this.#undo.canUndo();
		return fragment;
	}

	undo(): void {
		this.editing = null;
		this.#undo.undo();
	}

	redo(): void {
		this.editing = null;
		this.#undo.redo();
	}

	destroy(): void {
		this.#cleanup.forEach((stop) => stop());
		this.#undo.destroy();
		this.#room?.destroy();
	}
}

/** Opens deck `deckId`'s room and answers its editor; `destroy()` closes the room. */
export function createDeckEditor(deckId: string, user: DeckUser): DeckEditorState {
	const ydoc = new Y.Doc();
	const ref: { editor?: DeckEditorState } = {};
	const room = openDeckRoom(deckId, ydoc, {
		onconnection: (state) => ref.editor?.onconnection(state),
		onstop: (stop, reason) => ref.editor?.onstop(stop, reason),
		onsynced: () => ref.editor?.onsynced()
	});
	ref.editor = new DeckEditorState(new BoardDoc(ydoc), room.provider.awareness, user, room);
	return ref.editor;
}

/** A deck editor over a local document with no room, for tests and previews. */
export function localDeckEditor(ydoc = new Y.Doc(), user: DeckUser = { name: '' }): DeckEditorState {
	const editor = new DeckEditorState(new BoardDoc(ydoc), new Awareness(ydoc), user);
	editor.onsynced();
	editor.onconnection('live');
	return editor;
}
