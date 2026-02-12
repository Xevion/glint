import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.shaders.list();

	return result.match({
		Ok: (paginated) => ({
			shaders: paginated.items,
			total: paginated.total
		}),
		Err: (error) => ({
			shaders: [],
			total: 0,
			error: error.message
		})
	});
};
