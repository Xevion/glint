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

	const errors: string[] = [];

	const captures = capturesResult.match({
		Ok: (captures): Capture[] => captures,
		Err: (err) => {
			errors.push(err.message);
			return [] as Capture[];
		}
	});

	const shaders = shadersResult.match({
		Ok: (shaders): ShaderListItem[] => shaders,
		Err: (err) => {
			errors.push(err.message);
			return [] as ShaderListItem[];
		}
	});

	const featuredPairs = featuredResult.match({
		Ok: (pairs): FeaturedPair[] => pairs,
		Err: (err) => {
			errors.push(err.message);
			return [] as FeaturedPair[];
		}
	});

	return {
		captures,
		shaders,
		featuredPairs,
		error: errors.length > 0 ? errors.join('; ') : undefined
	};
};
