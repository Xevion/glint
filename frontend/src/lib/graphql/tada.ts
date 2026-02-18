import { initGraphQLTada } from 'gql.tada';
import type { introspection } from './graphql-env';

export const graphql = initGraphQLTada<{
	introspection: introspection;
	scalars: {
		DateTime: string;
		JSON: unknown;
		SceneId: string;
		ScenePresetId: string;
		SceneVersionId: string;
		ShaderVersionId: string;
		ShaderVersionProfileId: string;
		CaptureId: string;
		ShaderId: string;
		BackgroundId: string;
	};
}>();

export type { FragmentOf, ResultOf, VariablesOf } from 'gql.tada';
