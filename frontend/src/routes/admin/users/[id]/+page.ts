import { createApiClient, ApiErrorType } from '$lib/api';
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
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) error(404, { message: 'User not found' });
			error(500, { message: 'Failed to load user' });
		}
	});
};
