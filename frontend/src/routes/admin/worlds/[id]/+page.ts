import { createApiClient, ApiErrorType } from '$lib/api';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getWorld(params.id);

	return result.match({
		Ok: (data) => ({ world: data }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) error(404, { message: 'World not found' });
			error(500, { message: 'Failed to load world' });
		}
	});
};
