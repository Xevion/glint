import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import { ApiClient } from '../client';
import type {
	CaptureWithContext,
	PaginatedCaptures,
	Scene,
	SceneWithWorld,
	Shader,
	User,
	UserWithSessions,
	World
} from '$lib/bindings';

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

export class AdminEndpoints extends ApiClient {
	// ============== Shaders ==============

	listShaders(): Promise<Result<Shader[], ApiError>> {
		return super.get<Shader[]>('/api/shaders');
	}

	getShader(id: string): Promise<Result<Shader, ApiError>> {
		return super.get<Shader>(`/api/shaders/${id}`);
	}

	updateShader(id: string, request: UpdateShaderRequest): Promise<Result<Shader, ApiError>> {
		return super.put<Shader>(`/api/shaders/${id}`, request);
	}

	deleteShader(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/shaders/${id}`);
	}

	// ============== Worlds ==============

	getWorld(id: string): Promise<Result<World, ApiError>> {
		return super.get<World>(`/api/worlds/${id}`);
	}

	updateWorld(id: string, request: UpdateWorldRequest): Promise<Result<World, ApiError>> {
		return super.put<World>(`/api/worlds/${id}`, request);
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

	// ============== Health ==============

	health(): Promise<Result<string, ApiError>> {
		return this.getText('/health');
	}
}
