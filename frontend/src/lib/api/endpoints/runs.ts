import type { CaptureRun, CaptureRunItemWithContext } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class RunEndpoints extends ApiClient {
	list(): Promise<Result<CaptureRun[], ApiError>> {
		return this.get<CaptureRun[]>('/api/runs');
	}

	getById(id: string): Promise<Result<CaptureRun, ApiError>> {
		return this.get<CaptureRun>(`/api/runs/${encodeURIComponent(id)}`);
	}

	getItems(id: string): Promise<Result<CaptureRunItemWithContext[], ApiError>> {
		return this.get<CaptureRunItemWithContext[]>(`/api/runs/${encodeURIComponent(id)}/items`);
	}
}
