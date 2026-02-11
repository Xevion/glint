import { createApiClient } from '$lib/api';
import { ApiErrorType } from '$lib/api/errors';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch, url }) => {
	const api = createApiClient(fetch);
	const worldId = url.searchParams.get('world_id');

	const result = await api.scenes.getBySlug(params.id, worldId ?? undefined);

	return result.match({
		Ok: (scenes) => {
			if (scenes.length === 0) {
				error(404, { message: `Scene "${params.id}" not found` });
			}
			// Take the first scene (multi-world support can be added later)
			return { scene: scenes[0] };
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				error(404, { message: `Scene "${params.id}" not found` });
			}
			return err.throw();
		}
	});
};
