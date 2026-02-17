import type {
	CaptureWithContext,
	Paginated,
	SceneListItem,
	SceneWithCaptures
} from '$lib/bindings';
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
		if (worldId) searchParams.set('world_id', worldId);
		const query = searchParams.toString();
		const url = `/api/scenes/by-slug/${encodeURIComponent(slug)}${query ? `?${query}` : ''}`;
		return this.get<SceneWithCaptures[]>(url);
	}

	/**
	 * Paginated list of captures for a specific scene
	 */
	listCaptures(
		slug: string,
		params?: { page?: number; pageSize?: number }
	): Promise<Result<Paginated<CaptureWithContext>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		const qs = searchParams.toString();
		return this.get<Paginated<CaptureWithContext>>(
			`/api/scenes/by-slug/${encodeURIComponent(slug)}/captures${qs ? `?${qs}` : ''}`
		);
	}
}
