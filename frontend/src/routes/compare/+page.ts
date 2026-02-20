import { createGraphQLClient, graphql, query } from '$lib/graphql';
import type { ResultOf } from '$lib/graphql/tada';
import type { PageLoad } from './$types';

/** Lightweight query for the scene selector — only fields the dropdown needs. */
const CompareScenesQuery = graphql(`
	query CompareScenes($first: Int!) {
		scenes(first: $first) {
			edges {
				node {
					slug
					name
					captureCount
				}
			}
		}
	}
`);

/** Captures for the selected scene — only fields the comparison UI needs. */
const CompareCapturesQuery = graphql(`
	query CompareCaptures($id: String!) {
		scene(id: $id) {
			captures(first: 100) {
				edges {
					node {
						thumbhash
						shaderName
						shaderVersion
						shaderAuthor
						profileDisplayName
						presetName
						imagePath
					}
				}
			}
		}
	}
`);

type CompareScene = NonNullable<
	ResultOf<typeof CompareScenesQuery>['scenes']['edges'][number]['node']
>;
type CompareCapture = NonNullable<
	ResultOf<typeof CompareCapturesQuery>['scene']
>['captures']['edges'][number]['node'] & { imagePath: string };

interface ComparePageData {
	scenes: CompareScene[];
	captures: CompareCapture[];
	selectedSceneSlug: string | null;
	error: string | null;
}

export const load: PageLoad = async ({ fetch, url, depends }): Promise<ComparePageData> => {
	depends('glint:scenes');
	const client = createGraphQLClient(fetch);
	const sceneSlug = url.searchParams.get('scene') ?? undefined;

	const scenesResult = await query(client, CompareScenesQuery, { first: 100 });

	const scenes: CompareScene[] = scenesResult.match({
		Ok: (data) => data.scenes.edges.map((edge) => edge.node),
		Err: () => []
	});

	// Pick the target scene: explicit slug, or first scene with captures
	const targetSlug = sceneSlug ?? scenes.find((s) => s.captureCount > 0)?.slug ?? scenes[0]?.slug;

	if (!targetSlug) {
		return {
			scenes,
			captures: [],
			selectedSceneSlug: null,
			error: null
		};
	}

	const sceneResult = await query(client, CompareCapturesQuery, { id: targetSlug });

	return sceneResult.match({
		Ok: (data): ComparePageData => {
			const captures = (data.scene?.captures.edges.map((e) => e.node) ?? []).filter(
				(c): c is CompareCapture => !!c.imagePath
			);
			return {
				scenes,
				captures,
				selectedSceneSlug: targetSlug,
				error: null
			};
		},
		Err: (err): ComparePageData => ({
			scenes,
			captures: [],
			selectedSceneSlug: targetSlug,
			error: err.message
		})
	});
};
