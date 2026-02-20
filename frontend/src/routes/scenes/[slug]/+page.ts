import { createGraphQLClient, graphql, query } from '$lib/graphql';
import type { ResultOf } from '$lib/graphql/tada';
import { ApiErrorType, pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

const CAPTURES_PAGE_SIZE = 24;

const SceneDetailQuery = graphql(`
	query SceneDetail($id: String!) {
		scene(id: $id) {
			id
			name
			slug
			description
			dimension
			active
			createdAt
			imagePath
			thumbhash
			captureCount
			version {
				timeOfDayTicks
				weather
				weatherIntensity
				moonPhase
				biome
				fov
				renderDistance
			}
			presets {
				id
				name
				slug
				timeOfDayTicks
				weather
				weatherIntensity
				moonPhase
				sortOrder
			}
			captures {
				id
				shaderSlug
				shaderName
				shaderVersion
				shaderAuthor
				profileDisplayName
				imagePath
				thumbhash
				presetName
				freshness
			}
		}
	}
`);

type SceneResult = NonNullable<ResultOf<typeof SceneDetailQuery>['scene']>;

/** Capture type used by the scene detail page. */
export type SceneCapture = SceneResult['captures'][number];

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:scene:${params.slug}`);
	const client = createGraphQLClient(fetch);

	const result = await query(client, SceneDetailQuery, { id: params.slug });

	const sceneData = result.match({
		Ok: (data) => {
			if (!data.scene) {
				pageError(404, `Scene "${params.slug}" not found`);
			}
			return data.scene;
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				pageError(404, `Scene "${params.slug}" not found`);
			}
			return err.throw();
		}
	});

	const allCaptures = sceneData.captures;

	const scene = {
		id: sceneData.id,
		name: sceneData.name,
		slug: sceneData.slug,
		description: sceneData.description,
		dimension: sceneData.dimension,
		active: sceneData.active,
		createdAt: sceneData.createdAt,
		imagePath: sceneData.imagePath,
		thumbhash: sceneData.thumbhash,
		captureCount: sceneData.captureCount,
		version: sceneData.version,
		presets: sceneData.presets,
		captures: allCaptures
	};

	const capturesData = {
		items: allCaptures.slice(0, CAPTURES_PAGE_SIZE),
		total: allCaptures.length,
		page: 1,
		pageSize: CAPTURES_PAGE_SIZE
	};

	return { scene, capturesData };
};
