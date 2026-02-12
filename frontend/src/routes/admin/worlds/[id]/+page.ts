import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getWorld(params.id);

	return result.match({
		Ok: (data) => ({ world: data }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'World not found');
			return err.throw();
		}
	});
};
