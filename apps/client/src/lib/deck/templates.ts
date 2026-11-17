import { t } from '$lib/i18n';

const SLIDE = { w: 1920, h: 1080, gap: 200 };

type Text = { id: string; box: string; style: string; body: string; step?: number };

function text(shape: Text, z: string): string {
	const step = shape.step ? ` step="${shape.step}"` : '';
	return `\`\`\`text id="${shape.id}" ${shape.box} z="${z}"${step} ${shape.style}\n${shape.body}\n\`\`\``;
}

function frame(index: number, name: string, inside: string[]): string {
	const x = index * (SLIDE.w + SLIDE.gap);
	const head = `id="slide${index + 1}" x="${x}" y="0" w="${SLIDE.w}" h="${SLIDE.h}" name="${name}" order="a${index}"`;
	return `\`\`\`\`frame ${head}\n${inside.join('\n\n')}\n\`\`\`\``;
}

function cover(title: string): string {
	const heading = { id: 'title1', box: 'x="160" y="360" w="1600" h="220"', style: 'style.size="96"', body: `# ${title}` };
	const sub = { id: 'sub1', box: 'x="160" y="620" w="1600" h="120"', style: 'style.size="40"', body: t('starter.sub') };
	return frame(0, title, [text(heading, 'a0'), text(sub, 'a1')]);
}

function topic(): string {
	const name = t('starter.heading');
	const box = (y: number, h: number) => `x="160" y="${y}" w="1600" h="${h}"`;
	const shapes = [
		{ id: 'title2', box: box(120, 180), style: 'style.size="72"', body: `## ${name}` },
		{ id: 'point1', box: box(400, 120), style: 'style.size="44"', body: t('starter.first'), step: 1 },
		{ id: 'point2', box: box(560, 120), style: 'style.size="44"', body: t('starter.second'), step: 2 }
	];
	const notes = `\`\`\`notes\n${t('starter.note')}\n\`\`\``;
	return frame(1, name, [notes, ...shapes.map((shape, index) => text(shape, `a${index}`))]);
}

/**
 * The planche projection a new deck opens with, in the person's language: a cover, then one
 * slide that reveals two points and carries a speaker note. `title` goes in as plain text.
 */
export function starterSource(title: string): string {
	const plain = title.replace(/[\r\n"\\<>`*_[\]]/g, ' ').trim();
	return `---\nplanche: 1\nsize: ${SLIDE.w}x${SLIDE.h}\n---\n\n${cover(plain)}\n\n${topic()}\n`;
}
