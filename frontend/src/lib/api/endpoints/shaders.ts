import type { Paginated, ShaderListItem, ShaderWithCaptures } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export interface GetShaderParams {
	versionId?: string;
	profile_id?: string;
}

export class ShaderEndpoints extends ApiClient {
	/**
	 * Paginated list of all shaders with enrichment data
	 */
	list(params?: {
		page?: number;
		pageSize?: number;
	}): Promise<Result<Paginated<ShaderListItem>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		const qs = searchParams.toString();
		return this.get<Paginated<ShaderListItem>>(`/api/shaders${qs ? `?${qs}` : ''}`);
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
		if (params?.profile_id) searchParams.set('profile_id', params.profile_id);
		const query = searchParams.toString();
		const url = `/api/shaders/${encodeURIComponent(idOrSlug)}${query ? `?${query}` : ''}`;
		return this.get<ShaderWithCaptures>(url);
	}
}
