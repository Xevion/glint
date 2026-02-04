import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import { ApiClient } from '../client';
import type {
	CaptureWithContext,
	Job,
	Scene,
	SceneWithWorld,
	Shader,
	User,
	UserWithSessions,
	World
} from '$lib/bindings';

export type { Job };

export interface JobWithDetails extends Job {
	shader_name: string;
	shader_slug: string;
	shader_version: string;
	scene_count: number;
}

export interface CreateJobRequest {
	shader_version_id: string;
	scene_ids: string[];
	profiles?: string[];
	priority?: number;
}

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

	listCaptures(): Promise<Result<CaptureWithContext[], ApiError>> {
		return super.get<CaptureWithContext[]>('/api/captures/all');
	}

	getCapture(id: string): Promise<Result<CaptureWithContext, ApiError>> {
		return super.get<CaptureWithContext>(`/api/captures/${id}/details`);
	}

	deleteCapture(id: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/captures/${id}`);
	}

	// ============== Jobs ==============

	listJobs(): Promise<Result<JobWithDetails[], ApiError>> {
		return super.get<JobWithDetails[]>('/api/jobs');
	}

	createJob(request: CreateJobRequest): Promise<Result<Job, ApiError>> {
		return super.post<Job>('/api/jobs', request);
	}

	deleteJob(jobId: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/jobs/${jobId}`);
	}

	cancelJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/jobs/${jobId}/cancel`, {});
	}

	retryJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/jobs/${jobId}/retry`, {});
	}

	releaseJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/jobs/${jobId}/release`, {});
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
