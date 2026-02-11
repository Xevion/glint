import { createApiClient } from '$lib/api';
import type { Background } from '$lib/bindings';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.backgrounds.list();

	const backgrounds: Background[] = result.match({
		Ok: (bgs) => bgs,
		Err: () => []
	});

	return { backgrounds };
};
