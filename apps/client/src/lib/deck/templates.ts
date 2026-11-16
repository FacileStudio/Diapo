import { t } from '$lib/i18n';
import type { AppKey } from '$lib/i18n/messages';

/** A ready-made scene of the insert menu, written as `docs/format.md` writes its examples. */
export interface Template {
	id: 'cover' | 'bullets' | 'split' | 'raw';
	label: AppKey;
	/** The scene text, separator and frontmatter included. `image` is the picture a split scene shows. */
	scene: (image: string) => string;
}

const cover = () =>
	`---\nlayout: cover\n---\n\n# ${t('template.title')}\n\n${t('template.subtitle')}\n\n<!-- ${t('template.note')} -->\n`;

const bullets = () =>
	`---\n\n## ${t('template.title')}\n\n- ${t('template.first')} {step=1}\n- ${t('template.second')} {step=2}\n- ${t('template.third')} {step=3}\n`;

const split = (image: string) =>
	`---\nlayout: split\n---\n\n## ${t('template.title')}\n\n- ${t('template.first')} {step=1}\n- ${t('template.second')} {step=2}\n\n![](${image})\n`;

const raw = () =>
	[
		'---',
		'layout: raw',
		'duration: 6',
		'---',
		'',
		'<div class="clip" data-start="0" data-duration="6" style="display: grid; place-items: center; height: 100%">',
		`  <h1 class="animated-title">${t('template.title')}</h1>`,
		'</div>',
		'<script>',
		'  scene.timeline((tl) => tl.from(".animated-title", { opacity: 0, y: 40, duration: 0.8 }));',
		'</script>',
		''
	].join('\n');

/** The scenes the insert menu offers. */
export const TEMPLATES: Template[] = [
	{ id: 'cover', label: 'insert.cover', scene: cover },
	{ id: 'bullets', label: 'insert.bullets', scene: bullets },
	{ id: 'split', label: 'insert.split', scene: split },
	{ id: 'raw', label: 'insert.raw', scene: raw }
];

/** A 16:9 grey plate as a `data:` URI: the picture of a split scene until the deck has one. */
export const PLACEHOLDER_IMAGE =
	'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 16 9%22%3E%3Crect width=%2216%22 height=%229%22 fill=%22%23999%22/%3E%3C/svg%3E';
