import { createApiClient } from '$lib/api';
import type { User } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const result = await api.admin.listUsers();

	return result.match({
		Ok: (users) => ({ users, error: null as string | null }),
		Err: (err) => ({ users: [] as User[], error: err.message })
	});
};
