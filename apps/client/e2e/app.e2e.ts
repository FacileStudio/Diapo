import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';
import { api, appendSource, openPage, stageFrame, waitData, type Stack } from './pages';
import { startStack } from './stack';

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

const SECOND = '\n---\n\n## Deuxième\n\n- Un {step=1}\n- Deux {step=2}\n\n<!-- Ralentir ici. -->\n';

beforeAll(async () => {
	stack = await startStack();
	browser = await chromium.launch({ headless: true });
}, 120_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
});

const shown = async (page: Page, testid: string, selector: string) =>
	(await stageFrame(page, testid)).$eval(selector, (el) => getComputedStyle(el).visibility);

test('a deck is created from the list and opens in the editor', async () => {
	editor = await openPage(browser, stack, '/');
	await editor.getByTestId('new-deck').click();
	await editor.getByLabel('Titre').fill('Démo e2e');
	await editor.getByRole('button', { name: 'Créer' }).click();
	await editor.waitForURL(/\/decks\/[^/]+$/);
	id = new URL(editor.url()).pathname.split('/').pop() ?? '';
	await waitData(editor, 'save-status', 'status', 'clean');
	await waitData(editor, 'preview', 'scene', 0);
	expect(await editor.getByTestId('scene-count').textContent()).toContain('1 scène');
});

test('typing a second slide saves it and the preview follows the caret', async () => {
	await appendSource(editor, SECOND);
	await waitData(editor, 'save-status', 'status', 'clean');
	await waitData(editor, 'preview', 'scene', 1);
	await waitData(editor, 'preview', 'step', 3);
	expect(await (await stageFrame(editor, 'preview')).locator('.scene.is-active h2').textContent()).toBe('Deuxième');
	expect(await editor.getByTestId('scene-count').textContent()).toContain('2 scènes');
	expect(await editor.getByTestId('position').textContent()).toContain('Scène 2 sur 2');
	expect((await api<{ source: string }>(stack, `/decks/${id}`)).source).toContain('## Deuxième');
});

test('a source the parser refuses shows its line, and saves again once fixed', async () => {
	await appendSource(editor, '\n---\ncolour: red\n---\n\nFin\n');
	await waitData(editor, 'save-status', 'status', 'invalid');
	expect(await editor.getByTestId('scene-errors').textContent()).toMatch(/Ligne 1\d/);
	await editor.keyboard.press('Control+z');
	await waitData(editor, 'save-status', 'status', 'clean');
	expect(await editor.getByTestId('scene-errors').count()).toBe(0);
});

test('an outside write reloads a clean editor and stops a dirty one in a conflict', async () => {
	const theirs = await api<{ source: string; rev: number }>(stack, `/decks/${id}`);
	const body = { source: theirs.source.replace('Deuxième', 'Seconde'), rev: theirs.rev };
	const { rev } = await api<{ rev: number }>(stack, `/decks/${id}/source`, { method: 'PUT', body });
	await editor.locator('.cm-content').getByText('## Seconde').waitFor({ timeout: 10_000 });

	await editor.route('**/api/decks/*/source', (route) => route.abort());
	await appendSource(editor, '\n<!-- À moi. -->\n');
	await api(stack, `/decks/${id}/source`, { method: 'PUT', body: { source: body.source + '\nÀ eux.\n', rev } });
	await editor.getByTestId('conflict').waitFor({ timeout: 10_000 });
	await editor.unroute('**/api/decks/*/source');
	await editor.getByRole('button', { name: 'Garder la mienne et écraser' }).click();
	await waitData(editor, 'save-status', 'status', 'clean');
	const stored = (await api<{ source: string }>(stack, `/decks/${id}`)).source;
	expect([stored.includes('À moi.'), stored.includes('À eux.')]).toEqual([true, false]);
});

test('present mode steps through reveals and scenes, in step with the presenter view', async () => {
	await editor.getByTestId('present').click();
	await editor.waitForURL(/\/present$/);
	await waitData(editor, 'present', 'last', 3);
	await editor.keyboard.press('ArrowRight');
	await waitData(editor, 'present', 'step', 1);
	expect(await shown(editor, 'present', '.scene.is-active li[data-step="1"]')).toBe('hidden');
	await editor.keyboard.press('Space');
	await waitData(editor, 'present', 'step', 2);
	expect(await shown(editor, 'present', '.scene.is-active li[data-step="1"]')).toBe('visible');
	expect(await shown(editor, 'present', '.scene.is-active li[data-step="2"]')).toBe('hidden');
	await editor.getByTestId('zone-next').click();
	await waitData(editor, 'present', 'step', 3);

	const presenter = await editor.context().newPage();
	await presenter.goto(`${stack.base}/decks/${id}/presenter`);
	await waitData(presenter, 'presenter', 'step', 3);
	await presenter.getByTestId('notes').getByText('À moi.').waitFor({ timeout: 5_000 });
	await presenter.keyboard.press('Home');
	await waitData(editor, 'present', 'scene', 0);
	await editor.getByTestId('zone-next').click();
	await waitData(presenter, 'presenter', 'step', 1);
	await presenter.close();

	await editor.keyboard.press('Home');
	await waitData(editor, 'present', 'scene', 0);
	await editor.keyboard.press('Escape');
	await editor.waitForURL(/\/decks\/[^/]+$/);
});

test('a shared deck opens without a session, as a page and as an embed, until the link is turned off', async () => {
	await editor.getByTestId('share-open').click();
	await editor.getByTestId('share-create').click();
	const url = await editor.getByTestId('share-url').inputValue();
	const token = url.split('/p/')[1];
	expect(url).toBe(`${stack.base}/p/${token}`);
	expect(await editor.getByTestId('share-embed').inputValue()).toContain(`src="${stack.base}/embed/${token}"`);

	const context = await browser.newContext({ locale: 'fr-FR', viewport: { width: 1280, height: 720 } });
	const visitor = await context.newPage();
	const calls: string[] = [];
	visitor.on('request', (request) => calls.push(new URL(request.url()).pathname));
	await visitor.goto(url);
	await waitData(visitor, 'viewer', 'last', 3);
	await visitor.keyboard.press('ArrowRight');
	await visitor.getByTestId('zone-next').click();
	await waitData(visitor, 'viewer', 'step', 2);
	expect(await (await stageFrame(visitor, 'viewer')).locator('.scene.is-active h2').textContent()).toBe('Seconde');
	expect(await visitor.getByTestId('made-with').getAttribute('href')).toBe('/');

	await visitor.goto(`${stack.base}/embed/${token}`);
	await waitData(visitor, 'viewer', 'last', 3);
	expect(await visitor.getByTestId('viewer-open').getAttribute('href')).toBe(`/p/${token}`);
	expect(await visitor.getByTestId('made-with').count()).toBe(0);
	const own = calls.filter((path) => path.startsWith('/api/') && !path.startsWith('/api/public/'));
	expect(own).toEqual([]);

	await editor.getByTestId('share-revoke').click();
	await waitData(editor, 'share-dialog', 'status', 'off');
	await visitor.goto(url);
	await visitor.getByTestId('link-gone').getByText('Ce lien n’est plus disponible').waitFor({ timeout: 10_000 });
	await context.close();
	await editor.keyboard.press('Escape');
});

test('an HTML export downloads a file, and the formats the API cannot make yet say soon', async () => {
	await editor.getByTestId('export-open').click();
	await waitData(editor, 'export-pdf', 'state', 'soon');
	await waitData(editor, 'export-mp4', 'state', 'soon');
	await editor.getByTestId('export-html').getByRole('button').click();
	await waitData(editor, 'export-html', 'state', 'done');
	const [download] = await Promise.all([
		editor.waitForEvent('download'),
		editor.getByTestId('export-html').getByTestId('export-download').click()
	]);
	expect(download.suggestedFilename()).toMatch(/\.html$/);
	expect(await Bun.file(await download.path()).text()).toContain('Seconde');
	await editor.keyboard.press('Escape');
});
