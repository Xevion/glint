import { ApiErrorType, pageError } from '$lib/api/errors';
import { ShaderCardFragment, type ShaderCardShader } from '$lib/components/ShaderCard.svelte';
import { createGraphQLClient, graphql, query, type ResultOf } from '$lib/graphql';
import type { PageLoad } from './$types';

export const _ShaderDetailQuery = graphql(`
  query ShaderDetail($id: String!, $versionId: String, $profileId: String) {
    shader(id: $id) {
      id
      name
      slug
      description
      iconUrl
      websiteUrl
      modrinthId
      curseforgeId
      sourceUrl
      upstreamDownloads
      viewCount
      versions {
        version {
          id
          version
        }
        captureCount
      }
      authors {
        name
        url
      }
      captures(versionId: $versionId, profileId: $profileId) {
        id
        imagePath
        thumbhash
        sceneName
        profileDisplayName
        shaderVersion
        shaderName
      }
      profiles(versionId: $versionId) {
        id
        shaderVersionId
        name
        label
        displayName
        description
        sortOrder
      }
      metadata(versionId: $versionId) {
        shaderVersionId
        hasCustomTextures
        extractedAt
        pipelineFeatures
        irisFeaturesRequired
        irisFeaturesOptional
        settingsScreen
        filePaths
        dimensionSupport
      }
    }
  }
`);

const SimilarShadersQuery = graphql(
	`
    query SimilarShaders($first: Int!) {
      shaders(first: $first) {
        edges {
          node {
            ...ShaderCardFields
          }
        }
      }
    }
  `,
	[ShaderCardFragment]
);

type GqlShaderDetail = NonNullable<ResultOf<typeof _ShaderDetailQuery>['shader']>;
type GqlCapture = GqlShaderDetail['captures'][number];
type GqlVersion = GqlShaderDetail['versions'][number];
type GqlProfile = GqlShaderDetail['profiles'][number];
type GqlMetadata = NonNullable<GqlShaderDetail['metadata']>;

export interface ShaderDetail {
	id: string;
	name: string;
	slug: string;
	description?: string | null;
	iconUrl?: string | null;
	websiteUrl?: string | null;
	modrinthId?: string | null;
	curseforgeId?: string | null;
	sourceUrl?: string | null;
	upstreamDownloads?: number | null;
	viewCount: number;
	authors: ShaderDetailAuthor[];
	versions: ShaderDetailVersion[];
	captures: ShaderDetailCapture[];
	profiles: GqlProfile[];
	metadata?: ShaderDetailMetadata;
}

export interface ShaderDetailAuthor {
	name: string;
	url?: string;
}

export interface ShaderDetailVersion {
	id: string;
	version: string;
	captureCount: number;
}

export type ShaderDetailCapture = GqlCapture;

export interface ShaderDetailMetadata {
	shaderVersionId: string;
	hasCustomTextures?: boolean | null;
	extractedAt: string;
	pipelineFeatures?: Record<string, unknown>;
	irisFeaturesRequired?: string[];
	irisFeaturesOptional?: string[];
	settingsScreen?: Record<string, unknown>[];
	filePaths?: string[];
	dimensionSupport?: string[];
}

export interface CapturesPageData {
	items: ShaderDetailCapture[];
	total: number;
	page: number;
	pageSize: number;
}

/** Deduplicate authors by name, preferring entries that have a URL. */
function deduplicateAuthors(authors: { name: string; url: string | null }[]): ShaderDetailAuthor[] {
	const byName = new Map<string, ShaderDetailAuthor>();
	for (const a of authors) {
		const existing = byName.get(a.name);
		if (!existing || (!existing.url && a.url)) {
			byName.set(a.name, { name: a.name, url: a.url ?? undefined });
		}
	}
	return [...byName.values()];
}

function mapVersion(v: GqlVersion): ShaderDetailVersion {
	return {
		id: v.version.id,
		version: v.version.version,
		captureCount: v.captureCount
	};
}

function mapMetadata(m: GqlMetadata): ShaderDetailMetadata {
	return {
		shaderVersionId: m.shaderVersionId,
		hasCustomTextures: m.hasCustomTextures,
		extractedAt: m.extractedAt,
		pipelineFeatures: m.pipelineFeatures as Record<string, unknown> | undefined,
		irisFeaturesRequired: m.irisFeaturesRequired as string[] | undefined,
		irisFeaturesOptional: m.irisFeaturesOptional as string[] | undefined,
		settingsScreen: m.settingsScreen as Record<string, unknown>[] | undefined,
		filePaths: m.filePaths as string[] | undefined,
		dimensionSupport: m.dimensionSupport as string[] | undefined
	};
}

/** Map the full GraphQL response to the page-local ShaderDetail shape. */
export function _toShaderDetail(s: GqlShaderDetail): ShaderDetail {
	return {
		id: s.id,
		name: s.name,
		slug: s.slug,
		description: s.description,
		iconUrl: s.iconUrl,
		websiteUrl: s.websiteUrl,
		modrinthId: s.modrinthId,
		curseforgeId: s.curseforgeId,
		sourceUrl: s.sourceUrl,
		upstreamDownloads: s.upstreamDownloads,
		viewCount: s.viewCount,
		authors: deduplicateAuthors(s.authors),
		versions: s.versions.map(mapVersion),
		captures: s.captures,
		profiles: s.profiles,
		metadata: s.metadata ? mapMetadata(s.metadata) : undefined
	};
}

const CAPTURES_PAGE_SIZE = 24;

export const load: PageLoad = async ({ params, fetch, url }) => {
	const client = createGraphQLClient(fetch);

	const versionIdParam = url.searchParams.get('version_id') ?? undefined;
	const profileIdParam = url.searchParams.get('profile_id') ?? undefined;

	const result = await query(client, _ShaderDetailQuery, {
		id: params.slug,
		versionId: versionIdParam,
		profileId: profileIdParam
	});

	let shaderData = result.match({
		Ok: (data) => {
			if (!data.shader) {
				pageError(404, `Shader "${params.slug}" not found`);
			}
			return data.shader;
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				pageError(404, `Shader "${params.slug}" not found`);
			}
			return err.throw();
		}
	});

	// If the latest version has no captures but an older one does, re-fetch
	// with the correct versionId so SSR data matches the version the UI selects.
	if (shaderData.captures.length === 0 && !versionIdParam) {
		const versionWithCaptures = shaderData.versions.find((v) => v.captureCount > 0);
		if (versionWithCaptures) {
			const refetchResult = await query(client, _ShaderDetailQuery, {
				id: params.slug,
				versionId: versionWithCaptures.version.id,
				profileId: profileIdParam
			});
			refetchResult.match({
				Ok: (data) => {
					if (data.shader) {
						shaderData = data.shader;
					}
				},
				Err: () => {
					// Keep original data on refetch failure
				}
			});
		}
	}

	const shader = _toShaderDetail(shaderData);

	// GraphQL returns ALL captures for the version — do client-side slicing
	const allCaptures = shader.captures;
	const capturesData: CapturesPageData = {
		items: allCaptures.slice(0, CAPTURES_PAGE_SIZE),
		total: allCaptures.length,
		page: 1,
		pageSize: CAPTURES_PAGE_SIZE
	};

	// Similar shaders
	const listResult = await query(client, SimilarShadersQuery, { first: 30 });
	/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access -- gql.tada fragment types unresolvable by eslint */
	const similarShaders: ShaderCardShader[] = listResult.match({
		Ok: (data) =>
			(data.shaders.edges.map((e) => e.node) as ShaderCardShader[]).filter(
				(s) => s.slug !== params.slug
			),
		Err: () => []
	});
	/* eslint-enable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access */

	return { shader, capturesData, similarShaders };
};
