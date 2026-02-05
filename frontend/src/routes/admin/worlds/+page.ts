import { createApiClient } from '$lib/api';
import type { World } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.worlds.list();

	return result.match({
		Ok: (worlds) => ({ worlds, error: null as string | null }),
		Err: (err) => ({ worlds: [] as World[], error: err.message })
	});
};
