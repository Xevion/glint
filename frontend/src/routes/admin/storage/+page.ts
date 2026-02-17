import { createApiClient } from '$lib/api';
import type { StorageBucket, StorageStats } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [statsRes, growthRes] = await Promise.all([
		api.admin.storageStats(),
		api.admin.storageGrowth()
	]);

	const stats = statsRes.match({
		Ok: (v) => v,
		Err: () => null as StorageStats | null
	});

	const growth = growthRes.match({
		Ok: (v) => v,
		Err: () => [] as StorageBucket[]
	});

	return { stats, growth };
};
