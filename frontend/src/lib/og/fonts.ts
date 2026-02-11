import { read } from '$app/server';
import type { SatoriOptions } from 'satori';

// Import fonts as URLs — Vite copies them to build output.
// Satori only supports WOFF/TTF/OTF (not WOFF2).
import geistSansBlackUrl from '@fontsource/geist-sans/files/geist-sans-latin-900-normal.woff?url';
import interRegularUrl from '@fontsource/inter/files/inter-latin-400-normal.woff?url';
import interMediumUrl from '@fontsource/inter/files/inter-latin-500-normal.woff?url';

let cached: SatoriOptions['fonts'] | null = null;

/**
 * Load fonts for OG image text rendering.
 * Uses SvelteKit's read() to access fonts from build output,
 * which works across all deployment environments.
 *
 * Results are cached in-memory after first load.
 */
export async function loadOGFonts(): Promise<SatoriOptions['fonts']> {
	if (cached) return cached;

	let geistSansBlack: ArrayBuffer;
	let interRegular: ArrayBuffer;
	let interMedium: ArrayBuffer;

	try {
		[geistSansBlack, interRegular, interMedium] = await Promise.all([
			read(geistSansBlackUrl).arrayBuffer(),
			read(interRegularUrl).arrayBuffer(),
			read(interMediumUrl).arrayBuffer()
		]);
	} catch (err) {
		throw new Error(
			`Failed to load OG fonts from build output. Ensure @fontsource/geist-sans and @fontsource/inter are installed: ${err instanceof Error ? err.message : String(err)}`
		);
	}

	cached = [
		{ name: 'Geist Sans', data: geistSansBlack, weight: 900, style: 'normal' as const },
		{ name: 'Inter', data: interRegular, weight: 400, style: 'normal' as const },
		{ name: 'Inter', data: interMedium, weight: 500, style: 'normal' as const }
	];

	return cached;
}
