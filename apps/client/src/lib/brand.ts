import { registerIcons } from '@facile/muse';

/** The brand mark of Diapo: a slide on its stand. */
export const logoIcon = 'diapo:logo';

const screen = 'M3 6a3 3 0 0 1 3-3h12a3 3 0 0 1 3 3v8a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3z';
const lines =
	'M7.25 8a.75.75 0 0 1 .75-.75h8a.75.75 0 0 1 0 1.5H8A.75.75 0 0 1 7.25 8m0 4a.75.75 0 0 1 .75-.75h5a.75.75 0 0 1 0 1.5H8a.75.75 0 0 1-.75-.75';
const stand = 'M11.25 17h1.5v1.69l2.78 2.78a.75.75 0 1 1-1.06 1.06L12 20.06l-2.47 2.47a.75.75 0 0 1-1.06-1.06l2.78-2.78z';

const logoBody = `<path fill="currentColor" opacity=".5" d="${screen}"/><path fill="currentColor" d="${lines}"/><path fill="currentColor" d="${stand}"/>`;

registerIcons({ [logoIcon]: { body: logoBody, width: 24, height: 24 } });
