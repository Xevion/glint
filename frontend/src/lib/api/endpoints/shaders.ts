import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';
import type { Shader, ShaderWithCaptures } from '../types';

export class ShaderEndpoints extends ApiClient {
	/**
	 * List all shaders
	 */
	list(): Promise<Result<Shader[], ApiError>> {
		return super.get<Shader[]>('/api/shaders');
	}

	/**
	 * Get a single shader by slug with versions and captures
	 */
	getBySlug(slug: string): Promise<Result<ShaderWithCaptures, ApiError>> {
		return super.get<ShaderWithCaptures>(`/api/shaders/${encodeURIComponent(slug)}`);
	}
}
