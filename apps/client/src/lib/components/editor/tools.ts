import { registerIcons } from '@facile/muse';

const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"';
const glyph = (body: string) => ({ body: `<g ${stroke}>${body}</g>`, width: 24, height: 24 });

/** The toolbar and rail glyphs muse does not carry, drawn in Solar's line style. */
export const tools = {
	rect: 'diapo:rect',
	ellipse: 'diapo:ellipse',
	diamond: 'diapo:diamond',
	image: 'diapo:image',
	video: 'diapo:video',
	undo: 'diapo:undo',
	redo: 'diapo:redo',
	front: 'diapo:front',
	back: 'diapo:back'
} as const;

registerIcons({
	[tools.rect]: glyph('<rect x="4" y="6" width="16" height="12" rx="2"/>'),
	[tools.ellipse]: glyph('<ellipse cx="12" cy="12" rx="8.5" ry="6.5"/>'),
	[tools.diamond]: glyph('<path d="M12 3.5 20.5 12 12 20.5 3.5 12Z"/>'),
	[tools.image]: glyph(
		'<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><circle cx="8.5" cy="9.5" r="1.5"/>' +
			'<path d="m3.5 17 5-4.5 4 3.5 3-2.5 5 4"/>'
	),
	[tools.video]: glyph('<rect x="3" y="5.5" width="13" height="13" rx="2.5"/><path d="m16 10 5-3v10l-5-3"/>'),
	[tools.undo]: glyph('<path d="M9 14 4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/>'),
	[tools.redo]: glyph('<path d="m15 14 5-5-5-5"/><path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13"/>'),
	[tools.front]: glyph('<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 4H6a2 2 0 0 0-2 2v10"/>'),
	[tools.back]: glyph('<rect x="4" y="4" width="12" height="12" rx="2"/><path d="M20 8v10a2 2 0 0 1-2 2H8"/>')
});
