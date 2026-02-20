import type { Background, BackgroundUploadResponse, UpdateBackgroundRequest } from '$lib/bindings';
import type { Result } from 'true-myth';
import { ApiClient } from '../client';
import type { ApiError } from '../errors';

export class BackgroundEndpoints extends ApiClient {
	initiateUpload(
		filename: string,
		contentType?: string
	): Promise<Result<BackgroundUploadResponse, ApiError>> {
		return this.post<BackgroundUploadResponse>('/api/backgrounds', {
			filename,
			content_type: contentType
		});
	}

	confirmUpload(
		id: string,
		metadata: {
			thumbhash?: string;
			width?: number;
			height?: number;
			file_size_bytes?: number;
			content_type?: string;
		}
	): Promise<Result<Background, ApiError>> {
		return this.post<Background>(`/api/backgrounds/${encodeURIComponent(id)}/confirm`, metadata);
	}

	update(
		id: string,
		request: Partial<UpdateBackgroundRequest>
	): Promise<Result<Background, ApiError>> {
		const body: UpdateBackgroundRequest = {
			enabled: request.enabled ?? null,
			theme_mode: request.theme_mode ?? null,
			sort_order: request.sort_order ?? null
		};
		return this.put<Background>(`/api/backgrounds/${encodeURIComponent(id)}`, body);
	}

	remove(id: string): Promise<Result<null, ApiError>> {
		return this.delete(`/api/backgrounds/${encodeURIComponent(id)}`);
	}
}
