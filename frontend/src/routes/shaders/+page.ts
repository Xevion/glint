import type { ShaderListItem } from '$lib/bindings';
import { createApiClient } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	const api = createApiClient(fetch);

	const page = Number(url.searchParams.get('page') ?? '1');
	const pageSize = Number(url.searchParams.get('page_size') ?? '24');
	const q = url.searchParams.get('q') ?? undefined;
	const sort = url.searchParams.get('sort') ?? undefined;

	const result = await api.shaders.list({ page, pageSize, q, sort });

	return result.match<{
		shaders: ShaderListItem[];
		total: number;
		page: number;
		pageSize: number;
		q: string;
		sort: string;
		error: string | null;
	}>({
		Ok: (paginated) => ({
			shaders: paginated.items,
			total: paginated.total,
			page: paginated.page,
			pageSize: paginated.page_size,
			q: q ?? '',
			sort: sort ?? 'popular',
			error: null
		}),
		Err: (error) => ({
			shaders: [],
			total: 0,
			page: 1,
			pageSize: 24,
			q: q ?? '',
			sort: sort ?? 'popular',
			error: error.message
		})
	});
};
