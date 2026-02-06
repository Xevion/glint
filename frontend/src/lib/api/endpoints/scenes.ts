import type { SceneListItem, SceneWithCaptures } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class SceneEndpoints extends ApiClient {
	/**
	 * List all scenes with enrichment data
	 */
	list(): Promise<Result<SceneListItem[], ApiError>> {
		return super.get<SceneListItem[]>('/api/scenes');
	}

	/**
	 * Get scenes by slug with world and captures (returns array for multi-world support)
	 */
	getBySlug(slug: string, worldId?: string): Promise<Result<SceneWithCaptures[], ApiError>> {
		// TODO: backend uses snake_case query params (world_id) — migrate to camelCase (worldId)
		const url = worldId
			? `/api/scenes/by-slug/${encodeURIComponent(slug)}?world_id=${encodeURIComponent(worldId)}`
			: `/api/scenes/by-slug/${encodeURIComponent(slug)}`;
		return super.get<SceneWithCaptures[]>(url);
	}
}
