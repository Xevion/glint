import { createGraphQLClient, graphql, query } from '$lib/graphql';
import { formatNumber, formatVersion } from '$lib/utils/format';
import { rawImageUrl } from '$lib/utils/image';
import type { TextOverlayProps } from './text';

export type OgType = 'shader' | 'scene' | 'home' | 'shaders' | 'scenes' | 'compare';

interface OgImageData {
	/** URL of the screenshot to use as background (null = text-only) */
	imageUrl: string | null;
	/** Text overlay props */
	text: TextOverlayProps;
	/** Whether this is a fallback for a missing resource (should not be cached long-term) */
	fallback?: boolean;
}

// Focused OG queries — request only the fields needed for image generation.

const OgShaderQuery = graphql(`
	query OgShader($id: String!) {
		shader(id: $id) {
			name
			upstreamDownloads
			latestVersion
			captures(first: 1) {
				edges {
					node {
						imagePath
						shaderAuthor
						profileDisplayName
					}
				}
			}
		}
	}
`);

const OgSceneQuery = graphql(`
	query OgScene($id: String!) {
		scene(id: $id) {
			name
			imagePath
			captureCount
		}
	}
`);

const OgHomeQuery = graphql(`
	query OgHome($featuredCount: Int!) {
		featured(count: $featuredCount) {
			right {
				imagePath
			}
		}
	}
`);

const OgShadersListQuery = graphql(`
	query OgShadersList {
		shaders(first: 1) {
			edges {
				node {
					imagePath
				}
			}
		}
	}
`);

const OgScenesListQuery = graphql(`
	query OgScenesList {
		scenes(first: 100) {
			edges {
				node {
					imagePath
				}
			}
			totalCount
		}
	}
`);

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
	const client = createGraphQLClient(fetchFn);

	switch (type) {
		case 'shader':
			return fetchShaderOgData(client, params);
		case 'scene':
			return fetchSceneOgData(client, params);
		case 'home':
			return fetchHomeOgData(client);
		case 'shaders':
			return fetchShadersListOgData(client);
		case 'scenes':
			return fetchScenesListOgData(client);
		case 'compare':
			return fetchCompareOgData(client, params);
		default: {
			const _exhaustive: never = type;
			throw new Error(`Unknown OG type: ${String(_exhaustive)}`);
		}
	}
}

type GqlClient = ReturnType<typeof createGraphQLClient>;

async function fetchShaderOgData(client: GqlClient, slug: string): Promise<OgImageData> {
	const result = await query(client, OgShaderQuery, { id: slug });

	if (result.isErr || !result.value.shader) {
		return {
			imageUrl: null,
			text: { title: 'Shader Not Found' },
			fallback: true
		};
	}

	const shader = result.value.shader;
	const heroCapture = shader.captures.edges[0]?.node;

	const metaParts: string[] = [];
	if (heroCapture?.shaderAuthor) metaParts.push(`by ${heroCapture.shaderAuthor}`);
	if (shader.upstreamDownloads)
		metaParts.push(`${formatNumber(shader.upstreamDownloads)} downloads`);
	if (heroCapture?.profileDisplayName) metaParts.push(heroCapture.profileDisplayName);

	return {
		imageUrl: rawImageUrl(heroCapture?.imagePath),
		text: {
			title: shader.name,
			subtitle: shader.latestVersion ? formatVersion(shader.latestVersion) : undefined,
			meta: metaParts.length > 0 ? metaParts.join(' \u00b7 ') : undefined
		}
	};
}

async function fetchSceneOgData(client: GqlClient, slug: string): Promise<OgImageData> {
	const result = await query(client, OgSceneQuery, { id: slug });

	if (result.isErr || !result.value.scene) {
		return {
			imageUrl: null,
			text: { title: 'Scene Not Found' },
			fallback: true
		};
	}

	const scene = result.value.scene;

	return {
		imageUrl: rawImageUrl(scene.imagePath),
		text: {
			title: scene.name,
			meta:
				scene.captureCount > 0
					? `${scene.captureCount} shader screenshot${scene.captureCount !== 1 ? 's' : ''}`
					: undefined
		}
	};
}

async function fetchHomeOgData(client: GqlClient): Promise<OgImageData> {
	const result = await query(client, OgHomeQuery, { featuredCount: 6 });

	const imageUrl =
		result.isOk && result.value.featured.length > 0
			? rawImageUrl(result.value.featured[0]?.right.imagePath)
			: null;

	return {
		imageUrl,
		text: {
			title: 'Glint',
			subtitle: 'Shader Preview Catalog',
			meta: 'Browse, compare, and discover Minecraft shaders'
		}
	};
}

async function fetchShadersListOgData(client: GqlClient): Promise<OgImageData> {
	const result = await query(client, OgShadersListQuery, {});

	const imageUrl =
		result.isOk && result.value.shaders.edges.length > 0
			? rawImageUrl(result.value.shaders.edges[0]?.node.imagePath)
			: null;

	return {
		imageUrl,
		text: {
			title: 'Shaders',
			meta: 'Browse and compare Minecraft shaders'
		}
	};
}

async function fetchScenesListOgData(client: GqlClient): Promise<OgImageData> {
	const result = await query(client, OgScenesListQuery, {});

	if (result.isErr) {
		return {
			imageUrl: null,
			text: {
				title: 'Scenes',
				meta: 'Minecraft test scenes for shader comparison'
			}
		};
	}

	const firstWithImage = result.value.scenes.edges.find((e) => e.node.imagePath != null);

	return {
		imageUrl: rawImageUrl(firstWithImage?.node.imagePath),
		text: {
			title: 'Scenes',
			meta: `${result.value.scenes.totalCount} test scenes for shader comparison`
		}
	};
}

async function fetchCompareOgData(client: GqlClient, sceneSlug: string): Promise<OgImageData> {
	if (!sceneSlug) {
		return {
			imageUrl: null,
			text: {
				title: 'Compare Shaders',
				meta: 'Side-by-side shader comparison for Minecraft'
			}
		};
	}

	const result = await query(client, OgSceneQuery, { id: sceneSlug });

	if (result.isErr || !result.value.scene) {
		return {
			imageUrl: null,
			text: {
				title: 'Compare Shaders',
				meta: 'Side-by-side shader comparison for Minecraft'
			},
			fallback: true
		};
	}

	const scene = result.value.scene;

	return {
		imageUrl: rawImageUrl(scene.imagePath),
		text: {
			title: 'Compare Shaders',
			subtitle: scene.name,
			meta: scene.captureCount > 0 ? `${scene.captureCount} shaders to compare` : undefined
		}
	};
}
