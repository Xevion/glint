import { createApiClient } from '$lib/api';
import type { Capture, ShaderListItem } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [capturesResult, shadersResult] = await Promise.all([
		api.captures.list(),
		api.shaders.list()
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

	return {
		captures,
		shaders,
		error: errors.length > 0 ? errors.join('; ') : undefined
	};
};
