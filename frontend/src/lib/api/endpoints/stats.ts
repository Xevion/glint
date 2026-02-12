import type { Stats } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class StatsEndpoints extends ApiClient {
	/**
	 * Get aggregate statistics (shader/scene/capture counts)
	 */
	getStats(): Promise<Result<Stats, ApiError>> {
		return this.get<Stats>('/api/stats');
	}
}
