import { createApiClient } from '$lib/api';
import { type ResultOf, createGraphQLClient, graphql, query } from '$lib/graphql';
import type { PageLoad } from './$types';

const AdminDashboardQuery = graphql(`
	query AdminDashboard($recentCount: Int!) {
		stats {
			shaderCount
			sceneCount
			captureCount
			userCount
			runCount
		}
		recentCaptures(count: $recentCount) {
			id
			imagePath
			thumbhash
			shaderName
			sceneName
			shaderVersion
			profileDisplayName
			resolutionWidth
			resolutionHeight
			fileSizeBytes
			capturedAt
		}
		captureHealth {
			completed
			missing
			stale
			failed
		}
	}
`);

const AdminStorageQuery = graphql(`
	query AdminStorage($days: Int!, $intervalHours: Int!) {
		storageStats {
			totalBytes
			captureCount
			avgBytes
			missingCount
		}
		storageGrowth(days: $days, intervalHours: $intervalHours) {
			date
			cumulativeBytes
			cumulativeCount
			bucketBytes
		}
	}
`);

export type RecentCapture = ResultOf<typeof AdminDashboardQuery>['recentCaptures'][number];
export type CaptureHealthData = NonNullable<ResultOf<typeof AdminDashboardQuery>['captureHealth']>;
export type StorageStatsData = NonNullable<ResultOf<typeof AdminStorageQuery>['storageStats']>;
export type StorageBucketData = ResultOf<typeof AdminStorageQuery>['storageGrowth'][number];

export const load: PageLoad = async ({ fetch, depends }) => {
	depends('glint:admin:dashboard');
	const client = createGraphQLClient(fetch);
	const api = createApiClient(fetch);

	const [dashboardRes, storageRes, healthRes] = await Promise.all([
		query(client, AdminDashboardQuery, { recentCount: 5 }),
		query(client, AdminStorageQuery, { days: 90, intervalHours: 1 }),
		api.admin.health()
	]);

	const errors: Record<string, string> = {};

	const dashboard = dashboardRes.match({
		Ok: (data) => ({
			shaderCount: data.stats.shaderCount,
			sceneCount: data.stats.sceneCount,
			captureCount: data.stats.captureCount,
			userCount: data.stats.userCount,
			runCount: data.stats.runCount,
			recentCaptures: data.recentCaptures,
			captureHealth: data.captureHealth
		}),
		Err: (e) => {
			errors.dashboard = e.message;
			return {
				shaderCount: 0,
				sceneCount: 0,
				captureCount: 0,
				userCount: 0,
				runCount: 0,
				recentCaptures: [] as RecentCapture[],
				captureHealth: null as CaptureHealthData | null
			};
		}
	});

	const storage = storageRes.match({
		Ok: (data) => ({
			storageStats: data.storageStats,
			storageGrowth: data.storageGrowth
		}),
		Err: (e) => {
			errors.storage = e.message;
			return {
				storageStats: null as StorageStatsData | null,
				storageGrowth: [] as StorageBucketData[]
			};
		}
	});

	const healthStatus = healthRes.match({
		Ok: () => 'ok' as const,
		Err: (e) => {
			errors.health = e.message;
			return 'error' as const;
		}
	});

	return {
		...dashboard,
		...storage,
		healthStatus,
		errors
	};
};
