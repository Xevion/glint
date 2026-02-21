import { filter, type FilterDescriptor } from '$lib/components/data-view';
import { type ResultOf, graphql } from '$lib/graphql';

export const AdminCapturesQuery = graphql(`
	query AdminCaptures($first: Int!, $after: String, $sort: String, $filters: AdminCaptureFiltersInput) {
		adminCaptures(first: $first, after: $after, sort: $sort, filters: $filters) {
			edges {
				node {
					id
					sceneId
					shaderSlug
					shaderName
					shaderVersion
					profileDisplayName
					imagePath
					thumbhash
					capturedAt
					resolutionWidth
					resolutionHeight
					fileSizeBytes
					runId
					runStatus
					shaderAuthor
					sceneName
					sceneSlug
					freshness
				}
			}
			pageInfo {
				hasNextPage
				endCursor
			}
			totalCount
		}
	}
`);

export const SceneFiltersQuery = graphql(`
	query CaptureSceneFilters {
		scenes(first: 250, visibility: INCLUDE) {
			edges {
				node {
					id
					name
				}
			}
		}
	}
`);

export const ShaderFiltersQuery = graphql(`
	query CaptureShaderFilters {
		shaders(first: 100, sort: "name", visibility: INCLUDE) {
			edges {
				node {
					id
					slug
					name
				}
			}
		}
	}
`);

export type AdminCaptureNode = ResultOf<
	typeof AdminCapturesQuery
>['adminCaptures']['edges'][number]['node'];

export interface CaptureFilters extends Record<string, FilterDescriptor<unknown>> {
	shaderSlug: FilterDescriptor<string>;
	sceneId: FilterDescriptor<string>;
	statuses: FilterDescriptor<string[]>;
	freshness: FilterDescriptor<string[]>;
	capturedAfter: FilterDescriptor<string>;
	capturedBefore: FilterDescriptor<string>;
	minFileSize: FilterDescriptor<number>;
	maxFileSize: FilterDescriptor<number>;
}

export const captureFilterConfig: CaptureFilters = {
	shaderSlug: filter<string>(),
	sceneId: filter<string>(),
	statuses: filter<string[]>({ default: [], parse: (raw) => raw.split(',') }),
	freshness: filter<string[]>({ default: [], parse: (raw) => raw.split(',') }),
	capturedAfter: filter<string>(),
	capturedBefore: filter<string>(),
	minFileSize: filter<number>({ parse: Number }),
	maxFileSize: filter<number>({ parse: Number })
};

export const captureStatusOptions = [
	{ value: 'UPLOADING', label: 'Uploading' },
	{ value: 'COMPLETED', label: 'Completed' },
	{ value: 'FAILED', label: 'Failed' }
];

export const captureFreshnessOptions = [
	{ value: 'FRESH', label: 'Fresh' },
	{ value: 'STALE', label: 'Stale' },
	{ value: 'SUPERSEDED', label: 'Superseded' }
];
