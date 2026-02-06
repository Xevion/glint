import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import type { CaptureRun, CaptureRunItemWithContext } from '$lib/bindings';
import { ApiClient } from '../client';

export class RunEndpoints extends ApiClient {
	list(): Promise<Result<CaptureRun[], ApiError>> {
		return super.get<CaptureRun[]>('/api/runs');
	}

	getById(id: string): Promise<Result<CaptureRun, ApiError>> {
		return super.get<CaptureRun>(`/api/runs/${encodeURIComponent(id)}`);
	}

	getItems(id: string): Promise<Result<CaptureRunItemWithContext[], ApiError>> {
		return super.get<CaptureRunItemWithContext[]>(`/api/runs/${encodeURIComponent(id)}/items`);
	}
}
