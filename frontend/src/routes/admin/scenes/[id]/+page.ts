import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
import type { ScenePreset } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);

	const [sceneRes, capturesRes] = await Promise.all([
		api.admin.getScene(params.id),
		api.admin.listCaptures({ scene: params.id, pageSize: 8 })
	]);

	const scene = sceneRes.match({
		Ok: (s) => s,
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Scene not found');
			return err.throw();
		}
	});

	const capturesData = capturesRes.match({
		Ok: (c) => c,
		Err: (err) => err.throw()
	});

	const presetsRes = await api.admin.listPresets(scene.slug);
	const presets = presetsRes.match({
		Ok: (p) => p,
		Err: () => [] as ScenePreset[]
	});

	return { scene, captures: capturesData.items, captureCount: capturesData.total, presets };
};
