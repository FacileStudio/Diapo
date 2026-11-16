import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';
import { CATALOG } from './catalog';
import type { Stack } from './pages';
import { startStack } from './stack';

const WIDTHS = [360, 390, 768, 1024, 1440];
const PRICING_LINKS = 'section[aria-labelledby] > div.grid a';

let stack: Stack;
let browser: Browser;

beforeAll(async () => {
	stack = await startStack();
	browser = await chromium.launch({ headless: true });
}, 120_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
});

async function visitor(path: string, width: number, locale = 'fr'): Promise<Page> {
	const context = await browser.newContext({ locale, viewport: { width, height: 900 }, reducedMotion: 'reduce' });
	await context.addCookies([{ name: 'fc_locale', value: locale, url: stack.base }]);
	await context.route('https://api-ardoise.facile.studio/**', (route) => route.fulfill({ json: CATALOG }));
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	await page.locator('h1').waitFor({ timeout: 15_000 });
	return page;
}

const overflow = (page: Page) => page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);

test('a signed-out visitor gets the landing, and no page scrolls sideways at the five widths', async () => {
	for (const width of WIDTHS) {
		const landing = await visitor('/', width);
		await landing.locator(PRICING_LINKS).first().waitFor({ timeout: 15_000 });
		expect(await landing.locator('h1').textContent()).toContain('Écrivez vos diapositives');
		expect([width, await overflow(landing)]).toEqual([width, 0]);
		await landing.goto(`${stack.base}/downloads`);
		await landing.locator('h1').waitFor();
		expect([width, await overflow(landing)]).toEqual([width, 0]);
		await landing.context().close();
	}
}, 120_000);

test('every pricing button opens the sign-up page in the visitor locale, monthly and yearly', async () => {
	for (const locale of ['fr', 'de', 'en', 'es']) {
		const page = await visitor('/', 1440, locale);
		await page.locator(PRICING_LINKS).first().waitFor({ timeout: 15_000 });
		for (const interval of ['month', 'year']) {
			await page.locator(`input[type=radio][value=${interval}]`).first().check({ force: true });
			const hrefs = await page
				.locator(PRICING_LINKS)
				.evaluateAll((links) => links.map((link) => link.getAttribute('href')));
			expect(hrefs.length).toBeGreaterThanOrEqual(3);
			expect(new Set(hrefs)).toEqual(new Set([`https://sso.facile.studio/signup?lang=${locale}`]));
		}
		const hero = await page.locator('main a[href^="https://sso.facile.studio/signup"]').first().getAttribute('href');
		expect(hero).toBe(`https://sso.facile.studio/signup?lang=${locale}`);
		await page.context().close();
	}
}, 120_000);

test('no link of the public pages is dead inside the app', async () => {
	const page = await visitor('/', 1440);
	const hrefs = await page
		.locator('a[href^="/"]')
		.evaluateAll((links) => links.map((link) => link.getAttribute('href') ?? ''));
	for (const href of new Set(hrefs.map((value) => value.split('#')[0]).filter((value) => !value.startsWith('/api/')))) {
		const answer = await fetch(`${stack.base}${href}`);
		expect([href, answer.status]).toEqual([href, 200]);
	}
	await page.context().close();
});
