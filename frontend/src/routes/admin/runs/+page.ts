import { createApiClient } from '$lib/api';
import type { CaptureRun } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.runs.list();

	return result.match({
		Ok: (runs) => ({ runs, error: null as string | null }),
		Err: (err) => ({ runs: [] as CaptureRun[], error: err.message })
	});
};
