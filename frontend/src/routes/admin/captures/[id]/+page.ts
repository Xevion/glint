import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.getCapture(params.id);

	return result.match({
		Ok: (data) => ({ capture: data }),
		Err: (e) => e.throw()
	});
};
