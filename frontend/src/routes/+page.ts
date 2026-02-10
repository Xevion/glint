import { createApiClient } from '$lib/api';
import type { Capture, FeaturedPair, ShaderListItem } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [capturesResult, shadersResult, featuredResult] = await Promise.all([
		api.captures.list(),
		api.shaders.list(),
		api.featured.list()
	]);

	const captures = capturesResult.match({
		Ok: (captures): Capture[] => captures,
		Err: () => [] as Capture[]
	});

	const shaders = shadersResult.match({
		Ok: (shaders): ShaderListItem[] => shaders,
		Err: () => [] as ShaderListItem[]
	});

	const featuredPairs = featuredResult.match({
		Ok: (pairs): FeaturedPair[] => pairs,
		Err: () => [] as FeaturedPair[]
	});

	return {
		captures,
		shaders,
		featuredPairs
	};
};
