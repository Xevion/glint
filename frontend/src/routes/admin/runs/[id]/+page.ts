import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);

	const [runRes, itemsRes] = await Promise.all([
		api.runs.getById(params.id),
		api.runs.getItems(params.id)
	]);

	const run = runRes.match({
		Ok: (r) => r,
		Err: (e) => e.throw()
	});

	const items = itemsRes.match({
		Ok: (i) => i,
		Err: (e) => e.throw()
	});

	return { run, items };
};
