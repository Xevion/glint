import type { DeviceCodeStatus, DeviceConfirmResponse } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class DeviceEndpoints extends ApiClient {
	/**
	 * Get device code status (checks if code exists and is valid)
	 */
	getCodeStatus(userCode: string): Promise<Result<DeviceCodeStatus, ApiError>> {
		return this.get<DeviceCodeStatus>(`/api/device/code/${encodeURIComponent(userCode)}`);
	}

	/**
	 * Confirm device authorization (user authorizes the device)
	 */
	confirm(userCode: string): Promise<Result<DeviceConfirmResponse, ApiError>> {
		return this.post<DeviceConfirmResponse>('/api/device/confirm', { user_code: userCode });
	}
}
