import type { FullType, Shape } from '@facile/planche';

/** The fill of a sticky without one, as the compiler paints it. */
export const STICKY = '#fde68a';

const TEXT_TYPES = new Set(['sticky', 'text', 'rect', 'ellipse', 'diamond']);

/** Whether a shape of `type` draws a text body. */
export function hasText(type: FullType): boolean {
	return TEXT_TYPES.has(type);
}

/** The box declarations of a shape: place and size in frame pixels, turn and opacity, as the compiler writes them. */
export function boxStyle(shape: Shape<FullType>, size?: { w: number; h: number }): string {
	const out = [`left:${shape.x}px`, `top:${shape.y}px`, `width:${size?.w ?? shape.w}px`, `height:${size?.h ?? shape.h}px`];
	if (shape.rot) out.push(`transform:rotate(${shape.rot}rad)`);
	if (shape.opacity !== undefined) out.push(`opacity:${shape.opacity}`);
	return out.join(';');
}

function fillOf(shape: Shape<FullType>): string[] {
	const fill = shape.style.fill ?? (shape.type === 'sticky' ? STICKY : undefined);
	if (!fill) return [];
	return [shape.type === 'text' ? `color:${fill}` : `background:${fill}`];
}

function outline(shape: Shape<FullType>): string[] {
	const { stroke, width } = shape.style;
	if (!stroke || shape.type === 'text') return [];
	return [`outline:${width ?? 2}px solid ${stroke}`, 'outline-offset:0'];
}

/** The look of a text type: fill or colour, ellipse and diamond outlines, stroke, font, size and alignment. */
export function lookStyle(shape: Shape<FullType>): string {
	const out = [...fillOf(shape)];
	if (shape.type === 'ellipse') out.push('border-radius:50%');
	if (shape.type === 'diamond') out.push('clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%)');
	out.push(...outline(shape));
	if (shape.style.font) out.push(`font-family:${shape.style.font}`);
	if (shape.style.size !== undefined) out.push(`font-size:${shape.style.size}px`);
	if (shape.style.align) out.push(`text-align:${shape.style.align}`);
	return out.join(';');
}
