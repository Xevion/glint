import { createApiClient } from '$lib/api';
import type { JobWithDetails } from '$lib/api/endpoints/admin';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listJobs();

	return result.match({
		Ok: (jobs) => ({ jobs, error: null as string | null }),
		Err: (err) => ({ jobs: [] as JobWithDetails[], error: err.message })
	});
};
