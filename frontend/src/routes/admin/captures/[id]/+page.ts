import { createApiClient, ApiErrorType } from '$lib/api';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getCapture(params.id);

	return result.match({
		Ok: (data) => ({ capture: data }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) error(404, { message: 'Capture not found' });
			error(500, { message: 'Failed to load capture' });
		}
	});
};
