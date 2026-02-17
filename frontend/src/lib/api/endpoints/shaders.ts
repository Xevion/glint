import type {
	CaptureWithContext,
	Paginated,
	ShaderListItem,
	ShaderWithCaptures
} from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export interface GetShaderParams {
	versionId?: string;
	profileId?: string;
}

export class ShaderEndpoints extends ApiClient {
	/**
	 * Paginated list of all shaders with enrichment data
	 */
	list(params?: {
		page?: number;
		pageSize?: number;
		q?: string;
		sort?: string;
	}): Promise<Result<Paginated<ShaderListItem>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		if (params?.q) searchParams.set('q', params.q);
		if (params?.sort) searchParams.set('sort', params.sort);
		const qs = searchParams.toString();
		return this.get<Paginated<ShaderListItem>>(`/api/shaders${qs ? `?${qs}` : ''}`);
	}

	/**
	 * Paginated list of captures for a specific shader
	 */
	listCaptures(
		slug: string,
		params?: {
			page?: number;
			pageSize?: number;
			versionId?: string;
			profileId?: string;
		}
	): Promise<Result<Paginated<CaptureWithContext>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		if (params?.versionId) searchParams.set('version_id', params.versionId);
		if (params?.profileId) searchParams.set('profile_id', params.profileId);
		const qs = searchParams.toString();
		return this.get<Paginated<CaptureWithContext>>(
			`/api/shaders/${encodeURIComponent(slug)}/captures${qs ? `?${qs}` : ''}`
		);
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
		if (params?.profileId) searchParams.set('profile_id', params.profileId);
		const query = searchParams.toString();
		const url = `/api/shaders/${encodeURIComponent(idOrSlug)}${query ? `?${query}` : ''}`;
		return this.get<ShaderWithCaptures>(url);
	}
}
