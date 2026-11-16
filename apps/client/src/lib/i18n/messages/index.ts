import type { Catalogs, Message } from '@facile/muse';
import type { Row } from '../row';
import { decks } from './decks';
import { editor } from './editor';
import { show } from './show';

const rows = { ...decks, ...editor, ...show };

export type AppKey = keyof typeof rows;
type AppCatalog = Record<AppKey, Message>;

function column(index: number): AppCatalog {
	const entries = Object.entries(rows as Record<string, Row>).map(([key, row]) => [key, row[index]]);
	return Object.fromEntries(entries) as AppCatalog;
}

/** The app's catalogs: English is the typed base, and every row carries all four locales. */
export const catalogs: Catalogs<AppCatalog> = {
	en: column(0),
	fr: column(1),
	es: column(2),
	de: column(3)
};
