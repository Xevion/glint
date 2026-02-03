import type { Result } from 'true-myth';
import type { ApiError } from '../errors';
import { ApiClient } from '../client';
import type { Job } from '$lib/bindings';

export type { Job };

export interface JobWithDetails extends Job {
	shader_name: string;
	shader_slug: string;
	shader_version: string;
	scene_count: number;
}

export interface CreateJobRequest {
	shader_version_id: string;
	scene_ids: string[];
	profiles?: string[];
	priority?: number;
}

export class AdminEndpoints extends ApiClient {
	/**
	 * List all jobs with details
	 */
	listJobs(): Promise<Result<JobWithDetails[], ApiError>> {
		return super.get<JobWithDetails[]>('/api/admin/jobs');
	}

	/**
	 * Create a new job
	 */
	createJob(request: CreateJobRequest): Promise<Result<Job, ApiError>> {
		return super.post<Job>('/api/admin/jobs', request);
	}

	/**
	 * Delete a job
	 */
	deleteJob(jobId: string): Promise<Result<null, ApiError>> {
		return super.delete<null>(`/api/admin/jobs/${jobId}`);
	}

	/**
	 * Cancel a job (pending/claimed only)
	 */
	cancelJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/admin/jobs/${jobId}/cancel`, {});
	}

	/**
	 * Retry a failed job
	 */
	retryJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/admin/jobs/${jobId}/retry`, {});
	}

	/**
	 * Release a claimed/running job
	 */
	releaseJob(jobId: string): Promise<Result<Job, ApiError>> {
		return super.put<Job>(`/api/admin/jobs/${jobId}/release`, {});
	}

	/**
	 * Get API health status
	 */
	health(): Promise<Result<string, ApiError>> {
		return this.getText('/health');
	}
}
