import { createApiClient } from '$lib/api';
import type { CaptureWithContext } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listCaptures();

	return result.match({
		Ok: (captures) => ({ captures, error: null as string | null }),
		Err: (err) => ({ captures: [] as CaptureWithContext[], error: err.message })
	});
};
