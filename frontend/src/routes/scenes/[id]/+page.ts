import { createApiClient } from '$lib/api';
import { ApiErrorType } from '$lib/api/errors';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.scenes.getBySlug(params.id);

	return result.match({
		Ok: (scene) => ({ scene }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				error(404, { message: `Scene "${params.id}" not found` });
			}
			console.error('Failed to load scene:', err);
			error(500, { message: 'Failed to load scene data' });
		}
	});
};
