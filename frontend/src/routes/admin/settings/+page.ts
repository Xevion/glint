import { createApiClient } from '$lib/api';
import type { Background } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.backgrounds.listAll();

	return result.match({
		Ok: (backgrounds) => ({ backgrounds, error: null as string | null }),
		Err: (err) => ({ backgrounds: [] as Background[], error: err.message })
	});
};
