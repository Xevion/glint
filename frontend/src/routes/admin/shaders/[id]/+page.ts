import type { RelayConnection } from '$lib/components/data-view';
import { ApiErrorType, pageError } from '$lib/api/errors';
import { createGraphQLClient, query } from '$lib/graphql';
import {
	AdminCapturesQuery,
	type AdminCaptureNode,
	SceneFiltersQuery
} from '../../captures/queries';
import type { PageLoad } from './$types';
import { ShaderDetailQuery } from './queries';

interface FilterOption {
	value: string;
	label: string;
}

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:admin:shader:${params.id}`);
	const client = createGraphQLClient(fetch);

	// Fetch shader detail + scene filter options in parallel
	const [shaderResult, sceneFiltersResult] = await Promise.all([
		query(client, ShaderDetailQuery, { id: params.id }),
		query(client, SceneFiltersQuery, {})
	]);

	const shader = shaderResult.match({
		Ok: (data) => {
			if (!data.shader) {
				pageError(404, 'Shader not found');
			}
			return data.shader;
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Shader not found');
			return err.throw();
		}
	});

	// Sequential: need shader.slug for the captures filter
	const capturesResult = await query(client, AdminCapturesQuery, {
		first: 24,
		filters: { shaderSlug: shader.slug }
	});

	const emptyConnection: RelayConnection<AdminCaptureNode> = {
		edges: [],
		pageInfo: { hasNextPage: false },
		totalCount: 0
	};

	const captures = capturesResult.match({
		Ok: (data) => data.adminCaptures,
		Err: () => emptyConnection
	});

	const sceneOptions: FilterOption[] = sceneFiltersResult.match({
		Ok: (data) => data.scenes.edges.map((e) => ({ value: e.node.id, label: e.node.name })),
		Err: () => []
	});

	return { shader, captures, sceneOptions };
};
