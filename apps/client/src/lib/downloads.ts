/** The public repository: the source, and where the `diapo` binaries will be released. */
export const REPO_HREF = 'https://github.com/FacileStudio/Diapo';

/**
 * Whether a GitHub release carries the `diapo` binaries. While false, the landing says "soon"
 * and `/downloads` offers no file and no install command: a link that answers 404 is worse than
 * none. Turn it true in the commit that follows the first tagged release.
 */
export const CLI_RELEASED: boolean = false;
