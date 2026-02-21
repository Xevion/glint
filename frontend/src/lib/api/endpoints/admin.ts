import type {
	CaptureDetail,
	CaptureHealthResponse,
	CaptureWithContext,
	Paginated,
	Scene,
	ScenePreset,
	SceneWithVersion,
	ShaderListItem,
	StorageAuditResult,
	StorageBucket,
	StorageCleanupResult,
	StorageStats,
	UpdateSceneMetadataRequest,
	User,
	UserWithSessions,
	WorkItem
} from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

interface CreatePresetRequest {
	name: string;
	slug: string;
	time_of_day_ticks: number;
	weather: string;
	weather_intensity?: number;
	moon_phase?: number;
}

interface UpdatePresetRequest {
	name?: string;
	time_of_day_ticks?: number;
	weather?: string;
	weather_intensity?: number;
	moon_phase?: number;
}

interface ReorderPresetsRequest {
	preset_ids: string[];
}

export class AdminEndpoints extends ApiClient {
	listShaders(params?: {
		page?: number;
		pageSize?: number;
	}): Promise<Result<Paginated<ShaderListItem>, ApiError>> {
		const searchParams = new URLSearchParams();
		searchParams.set('all', 'true');
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		const qs = searchParams.toString();
		return this.get<Paginated<ShaderListItem>>(`/api/shaders${qs ? `?${qs}` : ''}`);
	}

	getScene(id: string): Promise<Result<SceneWithVersion, ApiError>> {
		return this.get<SceneWithVersion>(`/api/scenes/${encodeURIComponent(id)}`);
	}

	updateScene(id: string, request: UpdateSceneMetadataRequest): Promise<Result<Scene, ApiError>> {
		return this.put<Scene>(`/api/scenes/${encodeURIComponent(id)}`, request);
	}

	disableScene(id: string): Promise<Result<null, ApiError>> {
		return this.delete(`/api/scenes/${encodeURIComponent(id)}`);
	}

	reactivateScene(id: string): Promise<Result<Scene, ApiError>> {
		return this.put<Scene>(`/api/scenes/${encodeURIComponent(id)}/reactivate`, {});
	}

	listPresets(sceneSlug: string): Promise<Result<ScenePreset[], ApiError>> {
		return this.get<ScenePreset[]>(`/api/scenes/by-slug/${encodeURIComponent(sceneSlug)}/presets`);
	}

	createPreset(
		sceneSlug: string,
		request: CreatePresetRequest
	): Promise<Result<ScenePreset, ApiError>> {
		return this.post<ScenePreset>(
			`/api/scenes/by-slug/${encodeURIComponent(sceneSlug)}/presets`,
			request
		);
	}

	updatePreset(
		sceneSlug: string,
		presetSlug: string,
		request: UpdatePresetRequest
	): Promise<Result<ScenePreset, ApiError>> {
		return this.patch<ScenePreset>(
			`/api/scenes/by-slug/${encodeURIComponent(sceneSlug)}/presets/${encodeURIComponent(presetSlug)}`,
			request
		);
	}

	deletePreset(sceneSlug: string, presetSlug: string): Promise<Result<null, ApiError>> {
		return this.delete(
			`/api/scenes/by-slug/${encodeURIComponent(sceneSlug)}/presets/${encodeURIComponent(presetSlug)}`
		);
	}

	reorderPresets(sceneSlug: string, presetIds: string[]): Promise<Result<ScenePreset[], ApiError>> {
		return this.patch<ScenePreset[]>(
			`/api/scenes/by-slug/${encodeURIComponent(sceneSlug)}/presets/reorder`,
			{ preset_ids: presetIds } satisfies ReorderPresetsRequest
		);
	}

	listCaptures(params?: {
		page?: number;
		pageSize?: number;
		shader?: string;
		scene?: string;
		status?: string;
		runId?: string;
	}): Promise<Result<Paginated<CaptureWithContext>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		if (params?.shader) searchParams.set('shader', params.shader);
		if (params?.scene) searchParams.set('scene', params.scene);
		if (params?.status) searchParams.set('status', params.status);
		if (params?.runId) searchParams.set('run_id', params.runId);
		const qs = searchParams.toString();
		return this.get<Paginated<CaptureWithContext>>(`/api/captures/all${qs ? `?${qs}` : ''}`);
	}

	getCapture(id: string): Promise<Result<CaptureDetail, ApiError>> {
		return this.get<CaptureDetail>(`/api/captures/${encodeURIComponent(id)}/details`);
	}

	deleteCapture(id: string): Promise<Result<null, ApiError>> {
		return this.delete(`/api/captures/${encodeURIComponent(id)}`);
	}

	listUsers(): Promise<Result<User[], ApiError>> {
		return this.get<User[]>('/api/users');
	}

	getUser(id: number): Promise<Result<UserWithSessions, ApiError>> {
		return this.get<UserWithSessions>(`/api/users/${encodeURIComponent(id)}`);
	}

	updateUserRole(id: number, role: string): Promise<Result<User, ApiError>> {
		return this.put<User>(`/api/users/${encodeURIComponent(id)}/role`, {
			role
		});
	}

	deleteUserSessions(id: number): Promise<Result<null, ApiError>> {
		return this.delete(`/api/users/${encodeURIComponent(id)}/sessions`);
	}

	deleteSession(userId: number, tokenPrefix: string): Promise<Result<null, ApiError>> {
		return this.delete(
			`/api/users/${encodeURIComponent(userId)}/sessions/${encodeURIComponent(tokenPrefix)}`
		);
	}

	captureHealth(): Promise<Result<CaptureHealthResponse, ApiError>> {
		return this.get<CaptureHealthResponse>('/api/admin/capture-health');
	}

	workQueue(limit = 50): Promise<Result<WorkItem[], ApiError>> {
		const searchParams = new URLSearchParams();
		searchParams.set('dry_run', 'true');
		searchParams.set('limit', String(limit));
		return this.get<WorkItem[]>(`/api/work?${searchParams.toString()}`);
	}

	health(): Promise<Result<string, ApiError>> {
		return this.getText('/api/health');
	}

	storageStats(): Promise<Result<StorageStats, ApiError>> {
		return this.get<StorageStats>('/api/admin/storage/stats');
	}

	storageGrowth(days = 90, intervalHours = 1): Promise<Result<StorageBucket[], ApiError>> {
		const searchParams = new URLSearchParams();
		searchParams.set('days', String(days));
		searchParams.set('interval_hours', String(intervalHours));
		return this.get<StorageBucket[]>(`/api/admin/storage/growth?${searchParams.toString()}`);
	}

	storageAudit(): Promise<Result<StorageAuditResult, ApiError>> {
		return this.post<StorageAuditResult>('/api/admin/storage/audit', {});
	}

	storageCleanup(keys: string[]): Promise<Result<StorageCleanupResult, ApiError>> {
		return this.post<StorageCleanupResult>('/api/admin/storage/cleanup', { keys });
	}
}
