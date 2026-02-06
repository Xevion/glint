import type { AdoptPreviewResponse, Shader, ShaderSearchResponse } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class AdoptEndpoints extends ApiClient {
	search(
		query: string,
		limit?: number,
		offset?: number
	): Promise<Result<ShaderSearchResponse, ApiError>> {
		return super.post<ShaderSearchResponse>('/api/shaders/search', { query, limit, offset });
	}

	preview(url: string): Promise<Result<AdoptPreviewResponse, ApiError>> {
		return super.post<AdoptPreviewResponse>('/api/shaders/adopt/preview', { url });
	}

	adopt(url: string): Promise<Result<Shader, ApiError>> {
		return super.post<Shader>('/api/shaders/adopt', { url });
	}
}
