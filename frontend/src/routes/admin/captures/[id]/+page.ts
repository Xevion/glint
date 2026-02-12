import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getCapture(params.id);

	return result.match({
		Ok: (data) => ({ capture: data }),
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Capture not found');
			return err.throw();
		}
	});
};
