import { graphql } from '$lib/graphql/tada';

export const CaptureCompletedSubscription = graphql(`
	subscription CaptureCompleted {
		captureCompleted {
			captureId
			shaderVersionId
			sceneId
		}
	}
`);
