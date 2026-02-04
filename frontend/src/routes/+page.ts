import { createApiClient } from '$lib/api';
import type { CaptureWithContext, Shader } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [capturesResult, shadersResult] = await Promise.all([
		api.captures.list(),
		api.shaders.list()
	]);

	return {
		captures: capturesResult.match({
			Ok: (captures): CaptureWithContext[] => captures,
			Err: (): CaptureWithContext[] => []
		}),
		shaders: shadersResult.match({
			Ok: (shaders): Shader[] => shaders,
			Err: (): Shader[] => []
		})
	};
};
