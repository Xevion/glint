import { createApiClient, ApiErrorType } from '$lib/api';
import { error } from '@sveltejs/kit';
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
			if (err.type === ApiErrorType.NotFound) error(404, { message: 'Run not found' });
			error(500, { message: 'Failed to load run' });
		}
	});

	const items = itemsRes.match({
		Ok: (i) => i,
		Err: () => {
			error(500, { message: 'Failed to load run items' });
		}
	});

	return { run, items };
};
