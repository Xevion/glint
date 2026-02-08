import type {
	CaptureHealthResponse,
	CaptureWithContext,
	PaginatedCaptures,
	Scene,
	SceneWithWorld,
	Shader,
	ShaderWithCaptures,
	User,
	UserWithSessions,
	WorkItem,
	WorldWithScenes
} from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export interface UpdateShaderRequest {
	name?: string;
	description?: string;
	modrinth_id?: string;
	curseforge_id?: string;
	website_url?: string;
}

export interface UpdateWorldRequest {
	name?: string;
	description?: string;
}

export interface UpdateSceneMetadataRequest {
	name?: string;
	description?: string;
}

export interface UpdateUserRoleRequest {
	role: string;
}

export interface StorageStats {
	total_bytes: number;
	capture_count: number;
	avg_bytes: number;
	missing_count: number;
}

export interface StorageBucket {
	date: number; // unix timestamp in seconds
	cumulative_bytes: number;
	cumulative_count: number;
	bucket_bytes: number;
}

export class AdminEndpoints extends ApiClient {
	// ============== Shaders ==============

	listShaders(): Promise<Result<Shader[], ApiError>> {
		return super.get<Shader[]>('/api/shaders');
	}

	getShader(id: string): Promise<Result<ShaderWithCaptures, ApiError>> {
		return super.get<ShaderWithCaptures>(`/api/shaders/${id}`);
	}

	updateShader(id: string, request: UpdateShaderRequest): Promise<Result<Shader, ApiError>> {
		return super.put<Shader>(`/api/shaders/${id}`, request);
	}

	deleteShader(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/shaders/${id}`);
	}

	// ============== Worlds ==============

	getWorld(id: string): Promise<Result<WorldWithScenes, ApiError>> {
		return super.get<WorldWithScenes>(`/api/worlds/${id}`);
	}

	updateWorld(id: string, request: UpdateWorldRequest): Promise<Result<WorldWithScenes, ApiError>> {
		return super.put<WorldWithScenes>(`/api/worlds/${id}`, request);
	}

	deleteWorld(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/worlds/${id}`);
	}

	// ============== Scenes ==============

	listScenes(): Promise<Result<SceneWithWorld[], ApiError>> {
		return super.get<SceneWithWorld[]>('/api/scenes/all');
	}

	getScene(id: string): Promise<Result<Scene, ApiError>> {
		return super.get<Scene>(`/api/scenes/${id}`);
	}

	updateScene(id: string, request: UpdateSceneMetadataRequest): Promise<Result<Scene, ApiError>> {
		return super.put<Scene>(`/api/scenes/${id}`, request);
	}

	disableScene(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/scenes/${id}`);
	}

	reactivateScene(id: string): Promise<Result<Scene, ApiError>> {
		return super.put<Scene>(`/api/scenes/${id}/reactivate`, {});
	}

	// ============== Captures ==============

	listCaptures(params?: {
		page?: number;
		pageSize?: number;
		shader?: string;
		scene?: string;
		status?: string;
		runId?: string;
	}): Promise<Result<PaginatedCaptures, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		if (params?.shader) searchParams.set('shader', params.shader);
		if (params?.scene) searchParams.set('scene', params.scene);
		if (params?.status) searchParams.set('status', params.status);
		if (params?.runId) searchParams.set('run_id', params.runId);
		const qs = searchParams.toString();
		return super.get<PaginatedCaptures>(`/api/captures/all${qs ? `?${qs}` : ''}`);
	}

	getCapture(id: string): Promise<Result<CaptureWithContext, ApiError>> {
		return super.get<CaptureWithContext>(`/api/captures/${id}/details`);
	}

	deleteCapture(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/captures/${id}`);
	}

	// ============== Users ==============

	listUsers(): Promise<Result<User[], ApiError>> {
		return super.get<User[]>('/api/users');
	}

	getUser(id: number): Promise<Result<UserWithSessions, ApiError>> {
		return super.get<UserWithSessions>(`/api/users/${id}`);
	}

	updateUserRole(id: number, role: string): Promise<Result<User, ApiError>> {
		return super.put<User>(`/api/users/${id}/role`, { role });
	}

	deleteUserSessions(id: number): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/users/${id}/sessions`);
	}

	// ============== Sessions ==============

	deleteSession(token: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/sessions/${token}`);
	}

	// ============== Capture Health ==============

	captureHealth(): Promise<Result<CaptureHealthResponse, ApiError>> {
		return super.get<CaptureHealthResponse>('/api/admin/capture-health');
	}

	workQueue(limit = 50): Promise<Result<WorkItem[], ApiError>> {
		return super.get<WorkItem[]>(`/api/work?dry_run=true&limit=${limit}`);
	}

	// ============== Health ==============

	health(): Promise<Result<string, ApiError>> {
		return this.getText('/health');
	}

	// ============== Storage ==============

	storageStats(): Promise<Result<StorageStats, ApiError>> {
		return super.get<StorageStats>('/api/admin/storage/stats');
	}

	storageGrowth(days = 90, intervalHours = 1): Promise<Result<StorageBucket[], ApiError>> {
		return super.get<StorageBucket[]>(
			`/api/admin/storage/growth?days=${days}&interval_hours=${intervalHours}`
		);
	}
}
