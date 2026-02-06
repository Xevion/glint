import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.list();

	return result.match({
		Ok: (shaders) => ({
			shaders
		}),
		Err: (error) => ({
			shaders: [],
			error: error.message
		})
	});
};
