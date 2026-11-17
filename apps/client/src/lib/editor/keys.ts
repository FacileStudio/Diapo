/** What a key press on the stage asks for. */
export type StageCommand =
	| { kind: 'nudge'; dx: number; dy: number }
	| { kind: 'remove' | 'undo' | 'redo' | 'duplicate' | 'edit' | 'leave' | 'all' };

const ARROWS: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, -1],
	ArrowDown: [0, 1]
};

type Press = Pick<KeyboardEvent, 'key' | 'shiftKey' | 'metaKey' | 'ctrlKey' | 'altKey'>;

function shortcut(event: Press): StageCommand | null {
	const key = event.key.toLowerCase();
	if (key === 'z') return { kind: event.shiftKey ? 'redo' : 'undo' };
	if (key === 'y') return { kind: 'redo' };
	if (key === 'd') return { kind: 'duplicate' };
	if (key === 'a') return { kind: 'all' };
	return null;
}

/** The command of a key press: `editing` is true while a text box has the keyboard, which keeps only Escape. */
export function stageCommand(event: Press, editing: boolean): StageCommand | null {
	if (editing) return event.key === 'Escape' ? { kind: 'leave' } : null;
	if (event.altKey) return null;
	if (event.metaKey || event.ctrlKey) return shortcut(event);
	const arrow = ARROWS[event.key];
	if (arrow) return { kind: 'nudge', dx: arrow[0] * (event.shiftKey ? 10 : 1), dy: arrow[1] * (event.shiftKey ? 10 : 1) };
	if (event.key === 'Delete' || event.key === 'Backspace') return { kind: 'remove' };
	if (event.key === 'Enter') return { kind: 'edit' };
	if (event.key === 'Escape') return { kind: 'leave' };
	return null;
}
