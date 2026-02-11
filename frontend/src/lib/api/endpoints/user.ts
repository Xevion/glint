import type { User } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class UserEndpoints extends ApiClient {
	/** GET /api/user/me - Returns the current user, or an error if not authenticated */
	me(): Promise<Result<User, ApiError>> {
		return this.get<User>('/api/user/me');
	}
}
