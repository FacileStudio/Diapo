import type { HAlign } from './actions';
import type { Reorder } from './shapes';

/** What a key press on the stage asks for. */
export type StageCommand =
	| { kind: 'nudge'; dx: number; dy: number }
	| { kind: 'reorder'; where: Reorder }
	| { kind: 'align'; align: HAlign }
	| { kind: 'remove' | 'undo' | 'redo' | 'duplicate' | 'edit' | 'leave' | 'all' | 'help' };

const ARROWS: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, -1],
	ArrowDown: [0, 1]
};

const ALIGNS: Record<string, HAlign> = { l: 'left', e: 'center', r: 'right', j: 'justify' };

const SIMPLE: Record<string, StageCommand> = {
	y: { kind: 'redo' },
	d: { kind: 'duplicate' },
	a: { kind: 'all' }
};

type Press = Pick<KeyboardEvent, 'key' | 'shiftKey' | 'metaKey' | 'ctrlKey' | 'altKey'>;

function shortcut(event: Press): StageCommand | null {
	const key = event.key.toLowerCase();
	if (key === 'z') return { kind: event.shiftKey ? 'redo' : 'undo' };
	if (key === ']' || key === '}') return { kind: 'reorder', where: event.shiftKey ? 'front' : 'forward' };
	if (key === '[' || key === '{') return { kind: 'reorder', where: event.shiftKey ? 'back' : 'backward' };
	if (event.shiftKey && ALIGNS[key]) return { kind: 'align', align: ALIGNS[key] };
	return SIMPLE[key] ?? null;
}

function typing(event: Press): StageCommand | null {
	if (event.key === 'Escape') return { kind: 'leave' };
	const command = (event.metaKey || event.ctrlKey) && !event.altKey ? shortcut(event) : null;
	return command?.kind === 'align' ? command : null;
}

/** The command of a key press: `editing` is true while a text box has the keyboard, which keeps Escape and alignment. */
export function stageCommand(event: Press, editing: boolean): StageCommand | null {
	if (editing) return typing(event);
	if (event.altKey) return null;
	if (event.metaKey || event.ctrlKey) return shortcut(event);
	const arrow = ARROWS[event.key];
	if (arrow) return { kind: 'nudge', dx: arrow[0] * (event.shiftKey ? 10 : 1), dy: arrow[1] * (event.shiftKey ? 10 : 1) };
	if (event.key === 'Delete' || event.key === 'Backspace') return { kind: 'remove' };
	if (event.key === 'Enter') return { kind: 'edit' };
	if (event.key === 'Escape') return { kind: 'leave' };
	if (event.key === '?') return { kind: 'help' };
	return null;
}
