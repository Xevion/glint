import { ApiErrorType, createApiClient } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch, depends }) => {
	depends(`glint:admin:user:${params.id}`);
	const api = createApiClient(fetch);
	const id = parseInt(params.id, 10);

	if (Number.isNaN(id)) {
		pageError(400, 'Invalid user ID');
	}

	const result = await api.admin.getUser(id);

	return result.match({
		Ok: (user) => ({ user }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'User not found');
			return err.throw();
		}
	});
};
