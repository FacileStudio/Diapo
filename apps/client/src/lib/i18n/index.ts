import { createTranslator, LOCALE_COOKIE, resolveLocale, type Locale } from '@facile/muse';
import { catalogs } from './messages';

const SUPPORTED: readonly Locale[] = ['fr', 'en', 'es', 'de'];

let active: () => Locale = () => 'fr';

function cookieLocale(): string | null {
	const prefix = `${LOCALE_COOKIE}=`;
	const found = document.cookie.split('; ').find((part) => part.startsWith(prefix));
	return found ? decodeURIComponent(found.slice(prefix.length)) : null;
}

/** The first of `tags` (BCP 47, preferred first) that Diapo speaks, else French. */
export function pickLocale(tags: readonly (string | null)[]): Locale {
	for (const tag of tags) {
		const base = (tag ?? '').toLowerCase().split('-')[0] as Locale;
		if (SUPPORTED.includes(base)) return base;
	}
	return 'fr';
}

/** The locale of this browser: the one picked in the user menu, then the browser's languages, then French. */
export function browserLocale(): Locale {
	return pickLocale([cookieLocale(), ...navigator.languages]);
}

/** Persists the locale picked in the user menu for the next visit. */
export function rememberLocale(locale: Locale): void {
	document.cookie = `${LOCALE_COOKIE}=${locale}; path=/; max-age=31536000; samesite=lax`;
}

/** Points the translator and the formats at the locale `provideLocale` resolved. */
export function bindLocale(source: () => Locale): void {
	active = source;
}

/** The locale currently in force. */
export function locale(): Locale {
	return resolveLocale(active());
}

/** A day and a time, written the way the current locale writes them. */
export function formatMoment(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return '';
	return new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(date);
}

/** Translates an app message in the current locale. */
export const t = createTranslator(catalogs, () => active());
