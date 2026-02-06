import type { CaptureWithContext } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class CaptureEndpoints extends ApiClient {
	/**
	 * List all captures
	 */
	list(): Promise<Result<CaptureWithContext[], ApiError>> {
		return super.get<CaptureWithContext[]>('/api/captures');
	}

	/**
	 * Get a single capture by ID
	 */
	getById(id: string): Promise<Result<CaptureWithContext, ApiError>> {
		return super.get<CaptureWithContext>(`/api/captures/${encodeURIComponent(id)}`);
	}
}
