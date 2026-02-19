import { graphql, type ResultOf } from '$lib/graphql';

export const CaptureRunItemFragment = graphql(`
	fragment CaptureRunItemFields on CaptureRunItemNode {
		id
		runId
		shaderVersionId
		sceneId
		profileId
		profileName
		profileDisplayName
		presetId
		status
		captureId
		errorMessage
		errorLog
		durationMs
		startedAt
		completedAt
		shaderName
		shaderSlug
		shaderVersion
		sceneName
		imagePath
		thumbhash
	}
`);

export const AdminCaptureRunQuery = graphql(
	`
		query AdminCaptureRun($id: String!) {
			adminCaptureRun(id: $id) {
				id
				agentId
				startedAt
				completedAt
				status
				totalItems
				completedItems
				failedItems
				skippedItems
				items {
					...CaptureRunItemFields
				}
			}
		}
	`,
	[CaptureRunItemFragment]
);

export type AdminCaptureRunData = NonNullable<
	ResultOf<typeof AdminCaptureRunQuery>['adminCaptureRun']
>;
export type AdminCaptureRunItem = AdminCaptureRunData['items'][number];
