import { createApiClient } from '$lib/api';
import type { Background, User } from '$lib/bindings';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const [userResult, bgResult] = await Promise.all([api.user.me(), api.backgrounds.list()]);

	const user: User | null = userResult.match({
		Ok: (u) => u,
		Err: () => null
	});

	const backgrounds: Background[] = bgResult.match({
		Ok: (bgs) => bgs,
		Err: () => []
	});

	return { user, backgrounds };
};
