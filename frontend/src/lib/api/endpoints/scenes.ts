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
	 * Get scene by slug with captures
	 */
	getBySlug(slug: string): Promise<Result<SceneWithCaptures[], ApiError>> {
		return this.get<SceneWithCaptures[]>(`/api/scenes/by-slug/${encodeURIComponent(slug)}`);
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
