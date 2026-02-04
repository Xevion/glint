import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import { ApiClient } from '../client';
import type { DeviceCodeStatus, DeviceConfirmResponse } from '$lib/bindings';

export class DeviceEndpoints extends ApiClient {
	/**
	 * Get device code status (checks if code exists and is valid)
	 */
	getCodeStatus(userCode: string): Promise<Result<DeviceCodeStatus, ApiError>> {
		return super.get<DeviceCodeStatus>(`/api/device/code/${encodeURIComponent(userCode)}`);
	}

	/**
	 * Confirm device authorization (user authorizes the device)
	 */
	confirm(userCode: string): Promise<Result<DeviceConfirmResponse, ApiError>> {
		return super.post<DeviceConfirmResponse>('/api/device/confirm', { user_code: userCode });
	}
}
