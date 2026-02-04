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
	 * Get scenes by slug with world and captures (returns array for multi-world support)
	 */
	getBySlug(slug: string, worldId?: string): Promise<Result<SceneWithCaptures[], ApiError>> {
		const url = worldId
			? `/api/scenes/${encodeURIComponent(slug)}?world_id=${encodeURIComponent(worldId)}`
			: `/api/scenes/${encodeURIComponent(slug)}`;
		return super.get<SceneWithCaptures[]>(url);
	}
}
