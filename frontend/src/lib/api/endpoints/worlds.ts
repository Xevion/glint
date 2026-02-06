import type { CreateWorldUploadResponse, World } from '$lib/bindings';
import { Result } from 'true-myth';
import { ApiClient } from '../client';
import { ApiError, ApiErrorType } from '../errors';

export type { CreateWorldUploadResponse };

export interface CreateWorldUploadRequest {
	name: string;
	slug: string;
	description?: string;
	minecraft_version: string;
	file_hash: string;
	file_size_bytes: number;
}

export interface CompleteWorldUploadRequest {
	upload_id: string;
}

export interface UploadProgress {
	loaded: number;
	total: number;
	percentage: number;
}

export class WorldsEndpoints extends ApiClient {
	async list(): Promise<Result<World[], ApiError>> {
		return this.get<World[]>('/api/worlds');
	}

	async createWorldUpload(
		request: CreateWorldUploadRequest
	): Promise<Result<CreateWorldUploadResponse, ApiError>> {
		return this.post<CreateWorldUploadResponse>('/api/worlds', request);
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

	async completeWorldUpload(
		slug: string,
		request: CompleteWorldUploadRequest
	): Promise<Result<World, ApiError>> {
		return this.post<World>(`/api/worlds/${encodeURIComponent(slug)}/complete`, request);
	}
}
