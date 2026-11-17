import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';
import { api, openPage, stageFrame, waitData, waitSource, type Stack } from './pages';
import { startStack } from './stack';

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

const TYPED = 'Bonjour e2e';

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

const slides = (page: Page) => page.getByTestId('slide-thumb');

test('a deck is created from the list and opens in the slide editor, with no markdown in sight', async () => {
	editor = await openPage(browser, stack, '/');
	await editor.getByTestId('new-deck').click();
	await editor.getByLabel('Titre').fill('Démo e2e');
	await editor.getByRole('button', { name: 'Créer' }).click();
	await editor.waitForURL(/\/decks\/[^/]+$/);
	id = new URL(editor.url()).pathname.split('/').pop() ?? '';
	await waitData(editor, 'editor', 'synced', 'true');
	expect(await editor.getByTestId('slide-count').textContent()).toContain('2 diapositives');
	expect(await slides(editor).first().textContent()).toContain('Démo e2e');
	expect(await editor.locator('.cm-content, [data-testid="source-editor"]').count()).toBe(0);
	expect((await api<{ source: string }>(stack, `/decks/${id}`)).source).toMatch(/^---\nplanche: 1\n/);
});

test('every control of the editor header has the same height', async () => {
	const heights = await editor
		.locator('header')
		.first()
		.locator('a, button, input')
		.evaluateAll((controls) => controls.map((control) => Math.round(control.getBoundingClientRect().height)));
	expect(heights.length).toBeGreaterThanOrEqual(8);
	expect([...new Set(heights)]).toEqual([44]);
});

test('a new slide with a text box and a note persists across a reload, and the preview shows it', async () => {
	await slides(editor).nth(1).click();
	await editor.getByTestId('slide-add').click();
	expect(await editor.getByTestId('slide-count').textContent()).toContain('3 diapositives');
	expect(await slides(editor).nth(2).getAttribute('aria-current')).toBe('true');
	await editor.getByTestId('tool-text').click();
	await editor.locator('.stage-slide .ProseMirror[contenteditable="true"]').click();
	await editor.keyboard.type(TYPED);
	await editor.getByTestId('notes-editor').locator('.ProseMirror').click();
	await editor.keyboard.type('À moi.');
	await waitSource(stack, id, 'À moi.');
	expect(await waitSource(stack, id, TYPED)).toContain('order=');

	await editor.reload();
	await waitData(editor, 'editor', 'synced', 'true');
	expect(await editor.getByTestId('slide-count').textContent()).toContain('3 diapositives');
	expect(await slides(editor).nth(2).textContent()).toContain(TYPED);
	await slides(editor).nth(2).click();
	await editor.getByTestId('notes-editor').getByText('À moi.').waitFor({ timeout: 5_000 });

	await editor.getByTestId('preview-toggle').click();
	await waitData(editor, 'preview', 'scene', 2);
	await (await stageFrame(editor, 'preview')).getByText(TYPED).waitFor({ timeout: 10_000 });
	await editor.getByTestId('preview-toggle').click();
	await editor.getByTestId('toolbar').waitFor();
});

test('a slide moves with the keyboard and comes back', async () => {
	await slides(editor).nth(2).focus();
	await editor.keyboard.press('Alt+ArrowUp');
	await slides(editor).nth(1).getByText(TYPED).waitFor({ timeout: 5_000 });
	await slides(editor).nth(1).focus();
	await editor.keyboard.press('Alt+ArrowDown');
	await slides(editor).nth(2).getByText(TYPED).waitFor({ timeout: 5_000 });
});

test('a right-click duplicates a shape, and a text alignment persists across a reload', async () => {
	await slides(editor).first().click();
	const shapes = editor.locator('.stage-slide [data-shape]');
	const before = await shapes.count();
	await editor.getByTestId('tool-rect').click();
	await shapes.last().click({ button: 'right' });
	await editor.getByRole('menuitem', { name: /^Dupliquer/ }).click();
	await editor.waitForFunction((count) => document.querySelectorAll('.stage-slide [data-shape]').length === count, before + 2);
	await editor.getByTestId('text-align').getByRole('radio', { name: 'Justifier' }).click();
	await editor.getByTestId('text-valign').getByRole('radio', { name: 'Bas' }).click();
	await waitSource(stack, id, 'style.align="justify" style.valign="bottom"');

	await editor.reload();
	await waitData(editor, 'editor', 'synced', 'true');
	await slides(editor).first().click();
	const looks = await shapes.evaluateAll((nodes) =>
		nodes.map((node) => [getComputedStyle(node).textAlign, getComputedStyle(node).justifyContent].join(' '))
	);
	expect(looks.at(-1)).toBe('justify flex-end');
	await shapes.last().click({ button: 'right' });
	await editor.getByRole('menuitem', { name: /^Supprimer/ }).click();
	await shapes.last().click({ button: 'right' });
	await editor.getByRole('menuitem', { name: /^Supprimer/ }).click();
	await editor.waitForFunction((count) => document.querySelectorAll('.stage-slide [data-shape]').length === count, before);
});

test('two people on one deck see each other and each other’s edits live', async () => {
	const other = await openPage(browser, stack, `/decks/${id}`);
	await waitData(other, 'editor', 'synced', 'true');
	await editor.getByTestId('presence').getByRole('button', { name: 'Suivre Alice' }).waitFor({ timeout: 5_000 });
	await other.getByTestId('presence').getByRole('button', { name: 'Suivre Alice' }).waitFor({ timeout: 5_000 });

	await slides(editor).first().click();
	await slides(other).first().click();
	const shapes = (page: Page) => page.locator('.stage-slide [data-shape]');
	const before = await shapes(other).count();
	await editor.getByTestId('tool-rect').click();
	await other.waitForFunction(
		(count) => document.querySelectorAll('.stage-slide [data-shape]').length === count,
		before + 1,
		{ timeout: 1_000 }
	);
	await editor.getByTestId('tool-delete').click();
	await other.waitForFunction(
		(count) => document.querySelectorAll('.stage-slide [data-shape]').length === count,
		before,
		{ timeout: 1_000 }
	);
	expect(await shapes(editor).count()).toBe(before);
	await other.context().close();
	await editor.getByTestId('presence').waitFor({ state: 'detached', timeout: 10_000 });
});

test('present mode steps through reveals and scenes, in step with the presenter view', async () => {
	await editor.getByTestId('present').click();
	await editor.waitForURL(/\/present$/);
	await waitData(editor, 'present', 'last', 4);
	await editor.keyboard.press('ArrowRight');
	await waitData(editor, 'present', 'step', 1);
	expect(await shown(editor, 'present', '.scene.is-active [data-step="1"]')).toBe('hidden');
	await editor.keyboard.press('Space');
	await waitData(editor, 'present', 'step', 2);
	expect(await shown(editor, 'present', '.scene.is-active [data-step="1"]')).toBe('visible');
	expect(await shown(editor, 'present', '.scene.is-active [data-step="2"]')).toBe('hidden');
	await editor.getByTestId('zone-next').click();
	await waitData(editor, 'present', 'step', 3);

	const presenter = await editor.context().newPage();
	await presenter.goto(`${stack.base}/decks/${id}/presenter`);
	await waitData(presenter, 'presenter', 'step', 3);
	await presenter.getByTestId('notes').getByText('Vos notes vont ici.').waitFor({ timeout: 5_000 });
	await presenter.keyboard.press('End');
	await waitData(editor, 'present', 'step', 4);
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
	await waitData(visitor, 'viewer', 'last', 4);
	await visitor.keyboard.press('ArrowRight');
	await visitor.getByTestId('zone-next').click();
	await waitData(visitor, 'viewer', 'step', 2);
	await visitor.keyboard.press('End');
	await waitData(visitor, 'viewer', 'scene', 2);
	await (await stageFrame(visitor, 'viewer')).locator('.scene.is-active').getByText(TYPED).waitFor({ timeout: 5_000 });
	expect(await visitor.getByTestId('made-with').getAttribute('href')).toBe('/');

	await visitor.goto(`${stack.base}/embed/${token}`);
	await waitData(visitor, 'viewer', 'last', 4);
	expect(await visitor.getByTestId('viewer-open').getAttribute('href')).toBe(`/p/${token}`);
	expect(await visitor.getByTestId('made-with').count()).toBe(0);
	const own = calls.filter((path) => path.startsWith('/api/') && !path.startsWith('/api/public/'));
	expect(own).toEqual([]);

	await editor.getByTestId('share-revoke').click();
	await editor.getByRole('button', { name: 'Désactiver le lien' }).last().click();
	await waitData(editor, 'share-dialog', 'status', 'off');
	await visitor.goto(url);
	await visitor.getByTestId('link-gone').getByText('Ce lien n’est plus disponible').waitFor({ timeout: 10_000 });
	await context.close();
	await editor.keyboard.press('Escape');
	const dialog = editor.getByTestId('share-dialog');
	await dialog.waitFor({ state: 'hidden', timeout: 3000 }).catch(() => editor.keyboard.press('Escape'));
	await dialog.waitFor({ state: 'hidden', timeout: 5000 });
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
	expect(await Bun.file(await download.path()).text()).toContain(TYPED);
	await editor.keyboard.press('Escape');
});
