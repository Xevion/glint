import { createApiClient } from '$lib/api';
import type { SceneListAdmin } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listScenes();

	return result.match({
		Ok: (scenes) => ({ scenes, error: null as string | null }),
		Err: (err) => ({ scenes: [] as SceneListAdmin[], error: err.message })
	});
};
