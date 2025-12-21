import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import type { CaptureWithContext } from '../types';
import { ApiClient } from '../client';

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
