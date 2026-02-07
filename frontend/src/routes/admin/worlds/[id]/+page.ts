import { createApiClient } from '$lib/api';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getWorld(params.id);

	return result.match({
		Ok: (data) => ({ world: data }),
		Err: (e) => {
			error(e.statusCode ?? 500, e.message);
		}
	});
};
