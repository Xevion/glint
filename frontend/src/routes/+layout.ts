import { createApiClient } from '$lib/api';
import type { User } from '$lib/bindings';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.user.me();

	const user: User | null = result.match({
		Ok: (u) => u,
		Err: () => null
	});

	return { user };
};
