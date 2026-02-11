import type { SceneListItem, SceneWithCaptures } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class SceneEndpoints extends ApiClient {
	/**
	 * List all scenes with enrichment data
	 */
	list(): Promise<Result<SceneListItem[], ApiError>> {
		return this.get<SceneListItem[]>('/api/scenes');
	}

	/**
	 * Get scenes by slug with world and captures (returns array for multi-world support)
	 */
	getBySlug(slug: string, worldId?: string): Promise<Result<SceneWithCaptures[], ApiError>> {
		const searchParams = new URLSearchParams();
		if (worldId) searchParams.set('worldId', worldId);
		const query = searchParams.toString();
		const url = `/api/scenes/by-slug/${encodeURIComponent(slug)}${query ? `?${query}` : ''}`;
		return this.get<SceneWithCaptures[]>(url);
	}
}
