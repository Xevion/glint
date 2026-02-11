import type {
	CompleteUploadRequest,
	CreateWorldUploadRequest,
	CreateWorldVersionUploadRequest,
	UploadResponse,
	World,
	WorldListItem,
	WorldVersion
} from '$lib/bindings';
import { Result } from 'true-myth';
import { ApiClient } from '../client';
import { ApiError, ApiErrorType } from '../errors';

export type { UploadResponse };

export interface UploadProgress {
	loaded: number;
	total: number;
	percentage: number;
}

export class WorldsEndpoints extends ApiClient {
	list(): Promise<Result<WorldListItem[], ApiError>> {
		return this.get<WorldListItem[]>('/api/worlds');
	}

	createWorldUpload(request: CreateWorldUploadRequest): Promise<Result<UploadResponse, ApiError>> {
		return this.post<UploadResponse>('/api/worlds', request);
	}

	/**
	 * Upload file directly to R2 using presigned URL.
	 * Uses XHR for upload progress tracking — talks to R2, not our API.
	 */
	async uploadToPresignedUrl(
		presignedUrl: string,
		file: Blob,
		hash: string,
		onProgress?: (progress: UploadProgress) => void
	): Promise<Result<void, ApiError>> {
		return new Promise((resolve) => {
			const xhr = new XMLHttpRequest();

			xhr.upload.addEventListener('progress', (event) => {
				if (event.lengthComputable && onProgress) {
					onProgress({
						loaded: event.loaded,
						total: event.total,
						percentage: (event.loaded / event.total) * 100
					});
				}
			});

			xhr.addEventListener('load', () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					resolve(Result.ok(undefined));
				} else {
					resolve(
						Result.err(
							new ApiError(
								ApiErrorType.Unknown,
								`Upload failed with status ${xhr.status}: ${xhr.statusText}`,
								xhr.status
							)
						)
					);
				}
			});

			xhr.addEventListener('error', () => {
				resolve(Result.err(ApiError.network('Upload to storage failed')));
			});

			xhr.addEventListener('abort', () => {
				resolve(Result.err(ApiError.network('Upload was aborted')));
			});

			xhr.open('PUT', presignedUrl);
			xhr.setRequestHeader('Content-Type', 'application/zip');
			xhr.setRequestHeader('x-amz-meta-sha256', hash);
			xhr.send(file);
		});
	}

	completeWorldUpload(
		slug: string,
		request: CompleteUploadRequest
	): Promise<Result<World, ApiError>> {
		return this.post<World>(`/api/worlds/${encodeURIComponent(slug)}/complete`, request);
	}

	createWorldVersionUpload(
		worldId: string,
		request: CreateWorldVersionUploadRequest
	): Promise<Result<UploadResponse, ApiError>> {
		return this.post<UploadResponse>(`/api/worlds/${encodeURIComponent(worldId)}/versions`, request);
	}

	completeWorldVersionUpload(
		worldId: string,
		request: CompleteUploadRequest
	): Promise<Result<WorldVersion, ApiError>> {
		return this.post<WorldVersion>(
			`/api/worlds/${encodeURIComponent(worldId)}/versions/complete`,
			request
		);
	}
}
