import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';
import type { Scene, SceneWithCaptures } from '$lib/bindings';

export class SceneEndpoints extends ApiClient {
	/**
	 * List all scenes
	 */
	list(): Promise<Result<Scene[], ApiError>> {
		return super.get<Scene[]>('/api/scenes');
	}

	/**
	 * Get a single scene by slug with world and captures
	 */
	getBySlug(slug: string): Promise<Result<SceneWithCaptures, ApiError>> {
		return super.get<SceneWithCaptures>(`/api/scenes/${encodeURIComponent(slug)}`);
	}
}
