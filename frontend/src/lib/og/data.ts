import { createApiClient } from '$lib/api';
import { formatNumber, formatVersion } from '$lib/utils/display';
import type { TextOverlayProps } from './text';

export type OgType = 'shader' | 'scene' | 'home' | 'shaders';

export interface OgImageData {
	/** URL of the screenshot to use as background (null = text-only) */
	imageUrl: string | null;
	/** Text overlay props */
	text: TextOverlayProps;
}

/**
 * Fetch the data needed to render an OG image for the given type and params.
 *
 * @param fetchFn - SvelteKit's fetch function (for server-to-server API calls)
 */
export async function fetchOgData(
	type: OgType,
	params: string,
	fetchFn: typeof fetch
): Promise<OgImageData> {
	const api = createApiClient(fetchFn);

	switch (type) {
		case 'shader':
			return fetchShaderOgData(api, params);
		case 'scene':
			return fetchSceneOgData(api, params);
		case 'home':
			return fetchHomeOgData(api);
		case 'shaders':
			return fetchShadersListOgData(api);
		default: {
			const _exhaustive: never = type;
			throw new Error(`Unknown OG type: ${String(_exhaustive)}`);
		}
	}
}

type ApiClient = ReturnType<typeof createApiClient>;

async function fetchShaderOgData(api: ApiClient, slug: string): Promise<OgImageData> {
	const result = await api.shaders.getShader(slug);

	if (result.isErr) {
		return {
			imageUrl: null,
			text: { title: 'Shader Not Found' }
		};
	}

	const shader = result.value;
	const heroCapture = shader.captures[0];
	const latestVersion = shader.versions[0];

	const metaParts: string[] = [];
	if (heroCapture?.shader_author) metaParts.push(`by ${heroCapture.shader_author}`);
	if (shader.upstream_downloads)
		metaParts.push(`${formatNumber(shader.upstream_downloads)} downloads`);
	if (heroCapture?.profile) metaParts.push(heroCapture.profile);

	return {
		imageUrl: heroCapture?.image_url ?? null,
		text: {
			title: shader.name,
			subtitle: latestVersion ? formatVersion(latestVersion.version) : undefined,
			meta: metaParts.length > 0 ? metaParts.join(' \u00b7 ') : undefined
		}
	};
}

async function fetchSceneOgData(api: ApiClient, slug: string): Promise<OgImageData> {
	const result = await api.scenes.getBySlug(slug);

	if (result.isErr || result.value.length === 0) {
		return {
			imageUrl: null,
			text: { title: 'Scene Not Found' }
		};
	}

	const scene = result.value[0];
	const firstCapture = scene.captures[0];
	const captureCount = scene.captures.length;

	return {
		imageUrl: firstCapture?.image_url ?? null,
		text: {
			title: scene.name,
			meta:
				captureCount > 0
					? `${captureCount} shader screenshot${captureCount !== 1 ? 's' : ''}`
					: undefined
		}
	};
}

async function fetchHomeOgData(api: ApiClient): Promise<OgImageData> {
	const result = await api.featured.list();

	const imageUrl = result.isOk && result.value.length > 0 ? result.value[0].right_image_url : null;

	return {
		imageUrl,
		text: {
			title: 'Glint',
			subtitle: 'Shader Preview Catalog',
			meta: 'Browse, compare, and discover Minecraft shaders'
		}
	};
}

async function fetchShadersListOgData(api: ApiClient): Promise<OgImageData> {
	const result = await api.shaders.list();

	const imageUrl =
		result.isOk && result.value.length > 0 ? (result.value[0].image_url ?? null) : null;

	return {
		imageUrl,
		text: {
			title: 'Shaders',
			meta: 'Browse and compare Minecraft shaders'
		}
	};
}
