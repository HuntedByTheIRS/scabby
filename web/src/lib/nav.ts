/** Nav helpers. SvelteKit 2 types `page.url.pathname` as a route-pattern union,
 *  so comparisons go through a plain string parameter. */

export function isCurrent(pathname: string, href: string): boolean {
	return pathname === href || pathname.startsWith(href + '/');
}
