import type { Capture } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class CaptureEndpoints extends ApiClient {
	/**
	 * List completed captures (public)
	 */
	list(): Promise<Result<Capture[], ApiError>> {
		return this.get<Capture[]>('/api/captures');
	}

	/**
	 * Get a single capture by ID (public)
	 */
	getById(id: string): Promise<Result<Capture, ApiError>> {
		return this.get<Capture>(`/api/captures/${encodeURIComponent(id)}`);
	}
}
