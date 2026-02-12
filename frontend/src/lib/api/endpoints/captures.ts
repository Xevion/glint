import type { Capture, CaptureListItem, Paginated } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class CaptureEndpoints extends ApiClient {
	/**
	 * Paginated list of completed captures (public)
	 */
	list(params?: {
		page?: number;
		pageSize?: number;
	}): Promise<Result<Paginated<CaptureListItem>, ApiError>> {
		const searchParams = new URLSearchParams();
		if (params?.page != null) searchParams.set('page', String(params.page));
		if (params?.pageSize != null) searchParams.set('page_size', String(params.pageSize));
		const qs = searchParams.toString();
		return this.get<Paginated<CaptureListItem>>(`/api/captures${qs ? `?${qs}` : ''}`);
	}

	/**
	 * Get a single capture by ID (public)
	 */
	getById(id: string): Promise<Result<Capture, ApiError>> {
		return this.get<Capture>(`/api/captures/${encodeURIComponent(id)}`);
	}
}
