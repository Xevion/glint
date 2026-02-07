import { createApiClient } from '$lib/api';
import type { CaptureHealthResponse, WorkItem } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);
	const [healthRes, queueRes] = await Promise.all([
		api.admin.captureHealth(),
		api.admin.workQueue(50)
	]);

	const health = healthRes.match({
		Ok: (v) => v,
		Err: (e) => ({
			error: e.message,
			targets: [],
			summary: { total_targets: 0, completed: 0, missing: 0, stale: 0, failed: 0 }
		})
	});

	const workQueue = queueRes.match({
		Ok: (v) => v,
		Err: () => [] as WorkItem[]
	});

	return {
		health: health as CaptureHealthResponse & { error?: string },
		workQueue
	};
};
