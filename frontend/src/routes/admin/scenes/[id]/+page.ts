import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);

	const [sceneRes, capturesRes] = await Promise.all([
		api.admin.getScene(params.id),
		api.admin.listCaptures({ scene: params.id, pageSize: 8 })
	]);

	const scene = sceneRes.match({
		Ok: (s) => s,
		Err: (e) => e.throw()
	});

	const capturesData = capturesRes.match({
		Ok: (c) => c,
		Err: (e) => e.throw()
	});

	// Fetch world data for display name (non-critical — fall back to world_id)
	const world = scene.world_id
		? await api.admin.getWorld(scene.world_id).then((res) =>
				res.match({
					Ok: (w) => w,
					Err: () => null
				})
			)
		: null;

	return { scene, world, captures: capturesData.items, captureCount: capturesData.total };
};
