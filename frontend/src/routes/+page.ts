import { createApiClient } from '$lib/api';
import type { FeaturedPair, ShaderListItem, Stats } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [statsResult, shadersResult, featuredResult] = await Promise.all([
		api.stats.getStats(),
		api.shaders.list({ pageSize: 6 }),
		api.featured.list()
	]);

	const stats = statsResult.match({
		Ok: (s): Stats => s,
		Err: (): Stats => ({ shader_count: 0, scene_count: 0, capture_count: 0 })
	});

	const shaders = shadersResult.match({
		Ok: (result): ShaderListItem[] => result.items,
		Err: () => [] as ShaderListItem[]
	});

	const featuredPairs = featuredResult.match({
		Ok: (pairs): FeaturedPair[] => pairs,
		Err: () => [] as FeaturedPair[]
	});

	return {
		stats,
		shaders,
		featuredPairs
	};
};
