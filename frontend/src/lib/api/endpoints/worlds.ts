import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import { API_BASE_URL } from '../config';
import { ApiError as ApiErrorClass, ApiErrorType } from '../errors';
import { Result as TrueMythResult } from 'true-myth';
import type { World } from '../types';

export interface CreateWorldUploadRequest {
	name: string;
	slug: string;
	description?: string;
	minecraft_version: string;
	file_hash: string;
	file_size_bytes: number;
}

export interface CreateWorldUploadResponse {
	upload_id: string;
	presigned_url: string;
	expires_at: string;
}

export interface CompleteWorldUploadRequest {
	upload_id: string;
}

export interface UploadProgress {
	loaded: number;
	total: number;
	percentage: number;
}

export class WorldsEndpoints {
	private baseUrl: string;

	constructor(baseUrl: string = API_BASE_URL) {
		this.baseUrl = baseUrl;
	}

	/**
	 * List all worlds
	 */
	async listWorlds(): Promise<Result<World[], ApiError>> {
		const url = `${this.baseUrl}/api/admin/worlds`;

		try {
			const response = await fetch(url);

			if (!response.ok) {
				let body: unknown;
				try {
					body = await response.json();
				} catch {
					// Response body isn't JSON, ignore
				}
				return TrueMythResult.err(ApiErrorClass.fromResponse(response, body));
			}

			const data = (await response.json()) as World[];
			return TrueMythResult.ok(data);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Network request failed';
			return TrueMythResult.err(ApiErrorClass.network(message));
		}
	}

	/**
	 * Step 1: Request a presigned URL for world upload
	 */
	async createWorldUpload(
		request: CreateWorldUploadRequest
	): Promise<Result<CreateWorldUploadResponse, ApiError>> {
		const url = `${this.baseUrl}/api/admin/worlds`;

		try {
			const response = await fetch(url, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(request)
			});

			if (!response.ok) {
				let body: unknown;
				try {
					body = await response.json();
				} catch {
					// Response body isn't JSON
				}
				return TrueMythResult.err(ApiErrorClass.fromResponse(response, body));
			}

			const data = (await response.json()) as CreateWorldUploadResponse;
			return TrueMythResult.ok(data);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Network request failed';
			return TrueMythResult.err(ApiErrorClass.network(message));
		}
	}

	/**
	 * Step 2: Upload file directly to R2 using presigned URL
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
					resolve(TrueMythResult.ok(undefined));
				} else {
					resolve(
						TrueMythResult.err(
							new ApiErrorClass(
								ApiErrorType.Unknown,
								`Upload failed with status ${xhr.status}: ${xhr.statusText}`,
								xhr.status
							)
						)
					);
				}
			});

			xhr.addEventListener('error', () => {
				resolve(TrueMythResult.err(ApiErrorClass.network('Upload to storage failed')));
			});

			xhr.addEventListener('abort', () => {
				resolve(TrueMythResult.err(ApiErrorClass.network('Upload was aborted')));
			});

			xhr.open('PUT', presignedUrl);
			xhr.setRequestHeader('Content-Type', 'application/zip');
			// Set the hash metadata header (must match what backend expects)
			xhr.setRequestHeader('x-amz-meta-sha256', hash);
			xhr.send(file);
		});
	}

	/**
	 * Step 3: Complete the upload and create the world record
	 */
	async completeWorldUpload(
		slug: string,
		request: CompleteWorldUploadRequest
	): Promise<Result<World, ApiError>> {
		const url = `${this.baseUrl}/api/admin/worlds/${encodeURIComponent(slug)}/complete`;

		try {
			const response = await fetch(url, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(request)
			});

			if (!response.ok) {
				let body: unknown;
				try {
					body = await response.json();
				} catch {
					// Response body isn't JSON
				}
				return TrueMythResult.err(ApiErrorClass.fromResponse(response, body));
			}

			const data = (await response.json()) as World;
			return TrueMythResult.ok(data);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Network request failed';
			return TrueMythResult.err(ApiErrorClass.network(message));
		}
	}
}
