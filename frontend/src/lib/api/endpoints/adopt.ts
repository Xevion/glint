import type {
	AdoptPreviewResponse,
	Shader,
	ShaderSearchResponse,
	ShaderSearchSort
} from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class AdoptEndpoints extends ApiClient {
	search(
		query?: string,
		limit?: number,
		offset?: number,
		sort?: ShaderSearchSort
	): Promise<Result<ShaderSearchResponse, ApiError>> {
		return this.post<ShaderSearchResponse>('/api/shaders/search', {
			query: query ?? undefined,
			limit,
			offset,
			sort
		});
	}

	preview(url: string): Promise<Result<AdoptPreviewResponse, ApiError>> {
		return this.post<AdoptPreviewResponse>('/api/shaders/adopt/preview', { url });
	}

	adopt(url: string): Promise<Result<Shader, ApiError>> {
		return this.post<Shader>('/api/shaders/adopt', { url });
	}
}
