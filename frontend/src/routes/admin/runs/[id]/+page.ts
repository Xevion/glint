import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);

	const [runRes, itemsRes] = await Promise.all([
		api.runs.getById(params.id),
		api.runs.getItems(params.id)
	]);

	const run = runRes.match({
		Ok: (r) => r,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Run not found');
			return err.throw();
		}
	});

	const items = itemsRes.match({
		Ok: (i) => i,
		Err: (err) => err.throw()
	});

	return { run, items };
};
