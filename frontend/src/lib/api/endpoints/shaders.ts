import type { ShaderListItem, ShaderWithCaptures } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export interface GetShaderParams {
	versionId?: string;
	profile?: string;
}

export class ShaderEndpoints extends ApiClient {
	/**
	 * List all shaders with enrichment data
	 */
	list(): Promise<Result<ShaderListItem[], ApiError>> {
		return this.get<ShaderListItem[]>('/api/shaders');
	}

	/**
	 * Get a single shader by ID or slug, with versions and captures
	 */
	getShader(
		idOrSlug: string,
		params?: GetShaderParams
	): Promise<Result<ShaderWithCaptures, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.versionId) searchParams.set('version_id', params.versionId);
		if (params?.profile) searchParams.set('profile', params.profile);
		const query = searchParams.toString();
		const url = `/api/shaders/${encodeURIComponent(idOrSlug)}${query ? `?${query}` : ''}`;
		return this.get<ShaderWithCaptures>(url);
	}
}
