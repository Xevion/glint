import { createGraphQLClient, query } from '$lib/graphql';
import { ScenesQuery } from '$lib/graphql/queries/scenes';
import type { ResultOf } from '$lib/graphql/tada';
import type { PageLoad } from './$types';

type SceneNode = NonNullable<ResultOf<typeof ScenesQuery>['scenes']['edges'][number]['node']>;

export interface TrimmedScene {
	id: string;
	slug: string;
	name: string;
	description?: string;
	dimension: string;
	image_path?: string;
	thumbhash?: string;
	capture_count: number;
	version: {
		time_of_day_ticks: number;
		weather: string;
		biome?: string;
	};
}

function mapScene(node: SceneNode): TrimmedScene {
	return {
		id: node.id,
		slug: node.slug,
		name: node.name,
		description: node.description ?? undefined,
		dimension: node.dimension,
		image_path: node.imagePath ?? undefined,
		thumbhash: node.thumbhash ?? undefined,
		capture_count: node.captureCount,
		version: {
			time_of_day_ticks: node.version?.timeOfDayTicks ?? 0,
			weather: node.version?.weather ?? 'clear',
			biome: node.version?.biome ?? undefined
		}
	};
}

interface ScenesPageData {
	scenes: TrimmedScene[];
	error?: string;
}

export const load: PageLoad = async ({ fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, ScenesQuery, { first: 100 });

	return result.match<ScenesPageData>({
		Ok: (data) => ({
			scenes: data.scenes.edges.map((edge) => mapScene(edge.node))
		}),
		Err: (error) => ({
			scenes: [],
			error: error.message
		})
	});
};
