import type { FeaturedPair } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class FeaturedEndpoints extends ApiClient {
	/**
	 * Get random popular shader pairs for the homepage hero
	 */
	list(): Promise<Result<FeaturedPair[], ApiError>> {
		return this.get<FeaturedPair[]>('/api/featured');
	}
}
