import { createApiClient } from '$lib/api';
import type { FeaturedPair, ShaderListItem, Stats } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const errors: string[] = [];

	const [statsResult, shadersResult, featuredResult] = await Promise.all([
		api.stats.getStats(),
		api.shaders.list({ pageSize: 6 }),
		api.featured.list()
	]);

	const stats = statsResult.match({
		Ok: (s): Stats => s,
		Err: (e) => {
			errors.push(`Stats: ${e.message}`);
			return { shader_count: 0, scene_count: 0, capture_count: 0 } as Stats;
		}
	});

	const shaders = shadersResult.match({
		Ok: (result): ShaderListItem[] => result.items,
		Err: (e) => {
			errors.push(`Shaders: ${e.message}`);
			return [] as ShaderListItem[];
		}
	});

	const featuredPairs = featuredResult.match({
		Ok: (pairs): FeaturedPair[] => pairs,
		Err: (e) => {
			errors.push(`Featured: ${e.message}`);
			return [] as FeaturedPair[];
		}
	});

	return {
		stats,
		shaders,
		featuredPairs,
		errors
	};
};
