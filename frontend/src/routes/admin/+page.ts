import { createApiClient } from '$lib/api';
import type { StorageBucket, StorageStats } from '$lib/api/endpoints/admin';
import type { CaptureHealthSummary, CaptureWithContext } from '$lib/bindings';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const api = createApiClient(fetch);

	const [
		shadersRes,
		worldsRes,
		scenesRes,
		capturesRes,
		usersRes,
		healthRes,
		runsRes,
		captureHealthRes,
		storageStatsRes,
		storageGrowthRes
	] = await Promise.all([
		api.shaders.list(),
		api.worlds.list(),
		api.scenes.list(),
		api.admin.listCaptures(),
		api.admin.listUsers(),
		api.admin.health(),
		api.runs.list(),
		api.admin.captureHealth(),
		api.admin.storageStats(),
		api.admin.storageGrowth()
	]);

	const errors: Record<string, string> = {};

	const shaderCount = shadersRes.match({
		Ok: (v) => v.length,
		Err: (e) => {
			errors.shaders = e.message;
			return 0;
		}
	});

	const worldCount = worldsRes.match({
		Ok: (v) => v.length,
		Err: (e) => {
			errors.worlds = e.message;
			return 0;
		}
	});

	const sceneCount = scenesRes.match({
		Ok: (v) => v.length,
		Err: (e) => {
			errors.scenes = e.message;
			return 0;
		}
	});

	const { captureCount, recentCaptures } = capturesRes.match({
		Ok: (v) => ({ captureCount: v.total, recentCaptures: v.items.slice(0, 5) }),
		Err: (e) => {
			errors.captures = e.message;
			return { captureCount: 0, recentCaptures: [] as CaptureWithContext[] };
		}
	});

	const userCount = usersRes.match({
		Ok: (v) => v.length,
		Err: (e) => {
			errors.users = e.message;
			return 0;
		}
	});

	const runCount = runsRes.match({
		Ok: (v) => v.length,
		Err: (e) => {
			errors.runs = e.message;
			return 0;
		}
	});

	const healthStatus = healthRes.match({
		Ok: () => 'ok' as const,
		Err: (e) => {
			errors.health = e.message;
			return 'error' as const;
		}
	});

	const captureHealth = captureHealthRes.match({
		Ok: (v) => v.summary,
		Err: () => null as CaptureHealthSummary | null
	});

	const storageStats = storageStatsRes.match({
		Ok: (v) => v,
		Err: (e) => {
			errors.storage = e.message;
			return null as StorageStats | null;
		}
	});

	const storageGrowth = storageGrowthRes.match({
		Ok: (v) => v,
		Err: (e) => {
			errors.storageGrowth = e.message;
			return [] as StorageBucket[];
		}
	});

	return {
		shaderCount,
		worldCount,
		sceneCount,
		captureCount,
		userCount,
		runCount,
		recentCaptures,
		healthStatus,
		captureHealth,
		storageStats,
		storageGrowth,
		errors
	};
};
