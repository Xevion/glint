import { createApiClient } from '$lib/api';
import type { ShaderListItem } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listShaders({ pageSize: 250 });

	return result.match({
		Ok: (paginated) => ({ shaders: paginated.items, error: null as string | null }),
		Err: (err) => ({ shaders: [] as ShaderListItem[], error: err.message })
	});
};
