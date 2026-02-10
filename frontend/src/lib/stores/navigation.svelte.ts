import { beforeNavigate, onNavigate } from '$app/navigation';

export type NavDirection = 'left' | 'right';

/**
 * Path used for navbar label expansion. Deferred during view transitions so
 * CSS transitions run on visible DOM instead of snapping while hidden.
 * The pill animation (JS/RAF-driven) uses page.url.pathname directly.
 */
class NavbarState {
	path = $state(typeof window !== 'undefined' ? window.location.pathname : '/');
}

export const navigationStore = new NavbarState();

/** Route order for determining slide direction */
const ROUTE_ORDER = ['/', '/shaders', '/scenes', '/compare', '/admin', '/admin/settings'];

/**
 * Get route index for direction calculation.
 * Returns -1 for unknown routes (they'll get fade transition).
 */
function getRouteIndex(path: string): number {
	return ROUTE_ORDER.indexOf(path);
}

/**
 * Compute slide direction based on route order.
 * Forward in route order = slide left (old goes left, new comes from right)
 * Backward = slide right (old goes right, new comes from left)
 */
function computeDirection(from: string, to: string): NavDirection {
	const fromIdx = getRouteIndex(from);
	const toIdx = getRouteIndex(to);

	// If either route is unknown, default to left
	if (fromIdx === -1 || toIdx === -1) return 'left';

	// Forward in route order = slide left, backward = slide right
	return toIdx > fromIdx ? 'left' : 'right';
}

/** Call once from root layout to start tracking navigation direction */
export function initNavigation() {
	// Guard against SSR - this should only run in the browser
	if (typeof window === 'undefined') return;

	navigationStore.path = window.location.pathname;

	beforeNavigate(({ from, to }) => {
		if (!from?.url || !to?.url) return;
		const fromPath = from.url.pathname;
		const toPath = to.url.pathname;
		document.documentElement.dataset.navDirection = computeDirection(fromPath, toPath);
	});

	onNavigate((navigation) => {
		// Skip document-level view transitions for same-page navigations (e.g.
		// query param updates). Document transitions apply visibility:hidden to
		// the entire page, blocking all pointer interaction.
		const fromPath = navigation.from?.url.pathname;
		const toPath = navigation.to?.url.pathname;
		const isPageChange = fromPath !== toPath;

		if (!document.startViewTransition || !isPageChange) {
			void navigation.complete.then(() => {
				navigationStore.path = window.location.pathname;
			});
			return;
		}

		return new Promise((resolve) => {
			const vt = document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});

			// Update path only after the view transition finishes and the
			// real DOM is visible again, so CSS transitions can actually run.
			void vt.finished.finally(() => {
				navigationStore.path = window.location.pathname;
			});
		});
	});
}
