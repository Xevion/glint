import { createApiClient } from '$lib/api';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const id = parseInt(params.id, 10);

	if (Number.isNaN(id)) {
		error(400, 'Invalid user ID');
	}

	const result = await api.admin.getUser(id);

	return result.match({
		Ok: (user) => ({ user }),
		Err: (e) => {
			error(e.statusCode ?? 500, e.message);
		}
	});
};
