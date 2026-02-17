import { createApiClient, ApiErrorType } from '$lib/api';
import { pageError } from '$lib/api/errors';
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

	return { scene, captures: capturesData.items, captureCount: capturesData.total };
};
