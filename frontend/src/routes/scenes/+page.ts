import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.scenes.list();

	return result.match({
		Ok: (scenes) => ({
			scenes
		}),
		Err: (error) => ({
			scenes: [],
			error: error.message
		})
	});
};
