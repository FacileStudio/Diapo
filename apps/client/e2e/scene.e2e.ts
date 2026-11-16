import { afterAll, beforeAll, expect, test } from 'bun:test';
import { chromium, type Browser, type Page } from 'playwright-core';

type Contract = { ready: boolean; duration: number; steps: number; seek(t: number): void; step(k: number): void };
declare global {
	interface Window {
		__scene: Contract;
	}
}

const dir = process.env.SCENE_E2E_DIR ?? '';
let browser: Browser;

beforeAll(async () => {
	browser = await chromium.launch({ headless: true });
}, 60_000);

afterAll(async () => {
	await browser?.close();
});

async function open(file: string, width = 1920, height = 1080): Promise<Page> {
	const page = await browser.newPage({ viewport: { width, height } });
	await page.goto(`file://${dir}/${file}`);
	await page.waitForFunction(() => window.__scene?.ready === true);
	return page;
}

const style = (page: Page, selector: string, prop: string) =>
	page.$eval(selector, (el, name) => getComputedStyle(el).getPropertyValue(name as string), prop);

const inline = (page: Page, selector: string) => page.$eval(selector, (el) => (el as HTMLElement).style.cssText);

test('step(k) reveals the markers up to k and crosses scenes', async () => {
	const page = await open('steps.html');
	expect(await page.evaluate(() => [window.__scene.steps, window.__scene.duration])).toEqual([7, 16]);
	const second = 'section[data-index="0"] li[data-step="2"]';
	expect(await style(page, second, 'visibility')).toBe('hidden');
	await page.evaluate(() => window.__scene.step(2));
	expect(await style(page, second, 'visibility')).toBe('visible');
	await page.evaluate(() => window.__scene.step(1));
	expect(await style(page, second, 'visibility')).toBe('hidden');
	expect(await style(page, 'section[data-index="0"] li[data-step="1"]', 'visibility')).toBe('visible');
	await page.evaluate(() => window.__scene.step(5));
	expect(await page.$eval('.scene.is-active', (el) => (el as HTMLElement).dataset.index)).toBe('1');
	expect(await style(page, '#html-step', 'visibility')).toBe('hidden');
	expect(await style(page, 'section[data-index="1"] li[data-step="2"]', 'visibility')).toBe('visible');
	await page.close();
});

test('seek(t) maps time to scenes and steps, and the keyboard steps forward', async () => {
	const page = await open('steps.html', 960, 540);
	expect(await inline(page, '.stage')).toContain('scale(0.5)');
	await page.evaluate(() => window.__scene.seek(2.5));
	expect(await style(page, 'section[data-index="0"] li[data-step="1"]', 'visibility')).toBe('visible');
	expect(await style(page, 'section[data-index="0"] li[data-step="2"]', 'visibility')).toBe('hidden');
	await page.keyboard.press('ArrowRight');
	expect(await style(page, 'section[data-index="0"] li[data-step="2"]', 'visibility')).toBe('visible');
	await page.evaluate(() => window.__scene.seek(15));
	expect(await page.$eval('.scene.is-active', (el) => (el as HTMLElement).dataset.index)).toBe('2');
	await page.close();
});

test('a timeline seeks to the same frame whatever came before', async () => {
	const page = await open('clap.html', 1280, 720);
	expect(await page.evaluate(() => [window.__scene.steps, window.__scene.duration])).toEqual([2, 14]);
	const at = async (t: number) => {
		await page.evaluate((time) => window.__scene.seek(time), t);
		return inline(page, '#t');
	};
	expect(await at(0)).toBe('opacity: 0; transform: translate(0px, 40px) rotate(0deg) scale(1, 1);');
	expect(await at(0.5)).toBe('opacity: 0.5; transform: translate(0px, 20px) rotate(0deg) scale(1, 1);');
	const late = await at(4);
	expect(late).toBe('opacity: 1; transform: translate(0px, 0px) rotate(0deg) scale(1.5, 1.5);');
	await at(0.3);
	expect(await at(4)).toBe(late);
	expect(await style(page, '#sub', 'visibility')).toBe('visible');
	await at(1);
	expect(await style(page, '#sub', 'visibility')).toBe('hidden');
	await at(8);
	expect(await inline(page, '#a')).toBe('transform: translate(0px, 0px) rotate(0deg) scale(1, 1);');
	await page.close();
});

test('a single scene document answers the same contract and takes messages', async () => {
	const page = await open('steps-1.html');
	expect(await page.evaluate(() => [window.__scene.steps, window.__scene.duration])).toEqual([3, 8]);
	expect(await page.$$eval('.scene', (all) => all.length)).toBe(1);
	await page.evaluate(() => window.postMessage({ type: 'scene:step', k: 3 }, '*'));
	await page.waitForFunction(() => getComputedStyle(document.querySelector('#html-step')!).visibility === 'visible');
	await page.close();
});
