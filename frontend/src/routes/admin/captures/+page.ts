import {
	type RelayConnection,
	emptyConnection,
	extractFiltersFromUrl
} from '$lib/components/data-view';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import {
	type AdminCaptureNode,
	AdminCapturesQuery,
	SceneFiltersQuery,
	ShaderFiltersQuery,
	captureFilterConfig
} from './queries';

interface FilterOption {
	value: string;
	label: string;
}

export const load: PageLoad = async ({ fetch, url }) => {
	const gql = createGraphQLClient(fetch);
	const filters = extractFiltersFromUrl(url.searchParams, captureFilterConfig, 'filters');

	const [capturesRes, scenesRes, shadersRes] = await Promise.all([
		query(gql, AdminCapturesQuery, {
			first: 50,
			...filters
		}),
		query(gql, SceneFiltersQuery, {}),
		query(gql, ShaderFiltersQuery, {})
	]);

	const shaders: FilterOption[] = shadersRes.match({
		Ok: (data) => data.shaders.edges.map((e) => ({ value: e.node.slug, label: e.node.name })),
		Err: () => []
	});

	const scenes: FilterOption[] = scenesRes.match({
		Ok: (data) => data.scenes.edges.map((e) => ({ value: e.node.id, label: e.node.name })),
		Err: () => []
	});

	return capturesRes.match<{
		captures: RelayConnection<AdminCaptureNode>;
		shaders: FilterOption[];
		scenes: FilterOption[];
		error: string | null;
	}>({
		Ok: (data) => ({
			captures: data.adminCaptures,
			shaders,
			scenes,
			error: null
		}),
		Err: (err) => ({
			captures: emptyConnection<AdminCaptureNode>(),
			shaders,
			scenes,
			error: err.message
		})
	});
};
