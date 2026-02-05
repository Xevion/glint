import { createApiClient } from '$lib/api';
import type { CaptureWithContext, Shader } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [capturesResult, shadersResult] = await Promise.all([
		api.captures.list(),
		api.shaders.list()
	]);

	const errors: string[] = [];

	const captures = capturesResult.match({
		Ok: (captures): CaptureWithContext[] => captures,
		Err: (err) => {
			errors.push(err.message);
			return [] as CaptureWithContext[];
		}
	});

	const shaders = shadersResult.match({
		Ok: (shaders): Shader[] => shaders,
		Err: (err) => {
			errors.push(err.message);
			return [] as Shader[];
		}
	});

	return {
		captures,
		shaders,
		error: errors.length > 0 ? errors.join('; ') : undefined
	};
};
