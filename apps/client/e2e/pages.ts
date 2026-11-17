import type { Browser, Frame, Page } from 'playwright-core';

/** What a running stack hands the tests: the app's URL and the session cookie of the signed-in person, if it needs one. */
export type Stack = { base: string; cookie: string | null; stop: () => Promise<void> | void };

/** A French browser page at `path`, signed in. Pages opened from one context share a `BroadcastChannel`. */
export async function openPage(browser: Browser, stack: Stack, path: string): Promise<Page> {
	const context = await browser.newContext({ locale: 'fr-FR', viewport: { width: 1440, height: 900 } });
	if (stack.cookie) await context.addCookies([{ name: 'session', value: stack.cookie, url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

/** Calls the API as the signed-in person, with the CSRF header a browser write carries. */
export async function api<T>(stack: Stack, path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
	const answer = await fetch(`${stack.base}/api${path}`, {
		method: init.method ?? 'GET',
		headers: {
			...(stack.cookie ? { cookie: `session=${stack.cookie}` } : {}),
			'content-type': 'application/json',
			'x-facile-csrf': '1'
		},
		body: init.body === undefined ? undefined : JSON.stringify(init.body)
	});
	if (!answer.ok) throw new Error(`${path}: ${answer.status} ${await answer.text()}`);
	return (await answer.json()) as T;
}

/** Waits until the element `testid` carries `data-<name>="<value>"`. */
export async function waitData(page: Page, testid: string, name: string, value: string | number): Promise<void> {
	await page.locator(`[data-testid="${testid}"][data-${name}="${value}"]`).first().waitFor({ timeout: 15_000 });
}

/** The shown whole-deck frame inside the element `testid`: the sandboxed document the page drives. */
export async function stageFrame(page: Page, testid: string): Promise<Frame> {
	const handle = await page.locator(`[data-testid="${testid}"] iframe:not(.opacity-0)`).last().elementHandle();
	const frame = await handle?.contentFrame();
	if (!frame) throw new Error(`no compiled frame in ${testid}`);
	return frame;
}

/** Waits until the stored source of deck `id` holds `text`: the room saves on its own schedule. */
export async function waitSource(stack: Stack, id: string, text: string): Promise<string> {
	for (let attempt = 0; attempt < 100; attempt += 1) {
		const { source } = await api<{ source: string }>(stack, `/decks/${id}`);
		if (source.includes(text)) return source;
		await Bun.sleep(200);
	}
	throw new Error(`deck ${id} never stored "${text}"`);
}
