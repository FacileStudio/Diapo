import { describe, expect, it } from 'vitest';
import { bindLocale, pickLocale, t } from './index';
import { catalogs } from './messages';

describe('i18n', () => {
	it('carries every message in the four locales, with the same placeholders', () => {
		const names = (text: unknown) => (JSON.stringify(text).match(/\{\w+\}/g) ?? []).sort().join();
		for (const key of Object.keys(catalogs.en) as (keyof typeof catalogs.en)[]) {
			for (const locale of ['fr', 'es', 'de'] as const) {
				expect(catalogs[locale][key], `${locale} ${key}`).toBeTruthy();
				expect(names(catalogs[locale][key]), `${locale} ${key}`).toBe(names(catalogs.en[key]));
			}
		}
	});

	it('defaults to French and follows the bound locale, plurals included', () => {
		expect(t('decks.title')).toBe('Présentations');
		expect([pickLocale(['pt-BR', 'de-AT']), pickLocale(['ja', null])]).toEqual(['de', 'fr']);
		bindLocale(() => 'en');
		expect([t('strip.count', { count: 1 }), t('strip.count', { count: 3 })]).toEqual(['1 scene', '3 scenes']);
		bindLocale(() => 'fr');
	});
});
