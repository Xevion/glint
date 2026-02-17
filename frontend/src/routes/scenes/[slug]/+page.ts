import { createApiClient } from '$lib/api';
import { ApiErrorType, pageError } from '$lib/api/errors';
import type { PageLoad } from './$types';

const CAPTURES_PAGE_SIZE = 24;

export const load: PageLoad = async ({ params, fetch }) => {
	const api = createApiClient(fetch);

	const result = await api.scenes.getBySlug(params.slug);

	const scene = result.match({
		Ok: (scenes) => {
			if (scenes.length === 0) {
				pageError(404, `Scene "${params.slug}" not found`);
			}
			return scenes[0];
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) {
				pageError(404, `Scene "${params.slug}" not found`);
			}
			return err.throw();
		}
	});

	const capturesResult = await api.scenes.listCaptures(params.slug, {
		page: 1,
		pageSize: CAPTURES_PAGE_SIZE
	});

	const capturesData = capturesResult.match({
		Ok: (p) => ({ items: p.items, total: p.total, page: p.page, pageSize: p.page_size }),
		Err: () => ({
			items: scene.captures,
			total: scene.captures.length,
			page: 1,
			pageSize: CAPTURES_PAGE_SIZE
		})
	});

	return { scene, capturesData };
};
