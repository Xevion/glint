import { ApiErrorType, createApiClient } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { RelayConnection } from '$lib/components/data-view';
import type { ScenePreset } from '$lib/bindings';
import { createGraphQLClient, query } from '$lib/graphql';
import {
	AdminCapturesQuery,
	type AdminCaptureNode,
	ShaderFiltersQuery
} from '../../captures/queries';
import type { PageLoad } from './$types';

interface FilterOption {
	value: string;
	label: string;
}

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:admin:scene:${params.id}`);
	const api = createApiClient(fetch);
	const gql = createGraphQLClient(fetch);

	// Fetch scene, captures (GraphQL), and shader filter options in parallel
	const [sceneRes, capturesRes, shaderFiltersRes] = await Promise.all([
		api.admin.getScene(params.id),
		query(gql, AdminCapturesQuery, {
			first: 24,
			filters: { sceneId: params.id }
		}),
		query(gql, ShaderFiltersQuery, {})
	]);

	const scene = sceneRes.match({
		Ok: (s) => s,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Scene not found');
			return err.throw();
		}
	});

	// Sequential: presets need the scene slug
	const presetsRes = await api.admin.listPresets(scene.slug);
	const presets = presetsRes.match({
		Ok: (p) => p,
		Err: () => [] as ScenePreset[]
	});

	const emptyConnection: RelayConnection<AdminCaptureNode> = {
		edges: [],
		pageInfo: { hasNextPage: false },
		totalCount: 0
	};

	const captures = capturesRes.match({
		Ok: (data) => data.adminCaptures,
		Err: () => emptyConnection
	});

	const shaderOptions: FilterOption[] = shaderFiltersRes.match({
		Ok: (data) => data.shaders.edges.map((e) => ({ value: e.node.slug, label: e.node.name })),
		Err: () => []
	});

	return { scene, captures, shaderOptions, presets };
};
