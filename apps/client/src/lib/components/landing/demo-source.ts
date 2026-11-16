/** The lines every demo deck opens with: a cover scene. `title` is the deck's own title. */
export function coverLines(title: string, short = false): string[] {
	const body = [`# ${title}`, ''];
	return short ? body : ['---', 'layout: cover', '---', '', ...body];
}

/** The lines the demo types after the cover: a second scene with a heading and two points. */
export function sceneLines(heading: string, first: string, second: string): string[] {
	return ['---', '', `## ${heading}`, '', `- ${first}`, `- ${second}`];
}

/** What the preview draws for the typed lines: whether the scene exists yet, its heading and its points. */
export function typedScene(lines: string[]): { open: boolean; heading: string; bullets: string[] } {
	const heading = lines.find((line) => line.startsWith('## '))?.slice(3) ?? '';
	const bullets = lines.filter((line) => line.startsWith('- ')).map((line) => line.slice(2));
	return { open: lines[0] === '---', heading, bullets };
}
