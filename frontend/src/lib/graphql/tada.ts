import { initGraphQLTada } from 'gql.tada';
import type { introspection } from './graphql-env';

export const graphql = initGraphQLTada<{
	introspection: introspection;
	scalars: {
		DateTime: string;
		SceneId: string;
		ScenePresetId: string;
		SceneVersionId: string;
		ShaderVersionId: string;
		CaptureId: string;
	};
}>();

export type { FragmentOf, ResultOf, VariablesOf } from 'gql.tada';
