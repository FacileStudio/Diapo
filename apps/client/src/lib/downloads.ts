import type { CliCommand } from '@facile/muse';

/** The public repository: the source, and where the `diapo` binaries are released. */
export const REPO_HREF = 'https://github.com/FacileStudio/Diapo';

/** Every release of the `diapo` command: one archive per platform beside checksums.txt. */
export const RELEASES_HREF = `${REPO_HREF}/releases`;

/** The checksums of the newest release, which the installer verifies every archive against. */
export const CHECKSUMS_HREF = `${RELEASES_HREF}/latest/download/checksums.txt`;

/**
 * Whether a GitHub release carries the `diapo` binaries. While false, the landing says "soon"
 * and `/downloads` offers no file and no install command: a link that answers 404 is worse than
 * none. True since v0.2.0, the first release with binaries.
 */
export const CLI_RELEASED: boolean = true;

/** The suite installer, then the repository shim that bootstraps it. */
export const INSTALL: CliCommand[] = [
	{ label: 'facile', command: 'facile install diapo' },
	{ label: 'curl', command: 'curl -fsSL https://raw.githubusercontent.com/FacileStudio/Diapo/main/install.sh | bash' }
];
