import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';
import type { ShaderListItem, ShaderWithCaptures } from '$lib/bindings';

export class ShaderEndpoints extends ApiClient {
	/**
	 * List all shaders with enrichment data
	 */
	list(): Promise<Result<ShaderListItem[], ApiError>> {
		return super.get<ShaderListItem[]>('/api/shaders');
	}

	/**
	 * Get a single shader by ID or slug, with versions and captures
	 */
	getShader(idOrSlug: string): Promise<Result<ShaderWithCaptures, ApiError>> {
		return super.get<ShaderWithCaptures>(`/api/shaders/${encodeURIComponent(idOrSlug)}`);
	}
}
