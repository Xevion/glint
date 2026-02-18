/* eslint-disable */
/* prettier-ignore */

export type introspection_types = {
	Boolean: unknown;
	CaptureCompletedEvent: {
		kind: 'OBJECT';
		name: 'CaptureCompletedEvent';
		fields: {
			captureId: {
				name: 'captureId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'CaptureId'; ofType: null };
				};
			};
			sceneId: {
				name: 'sceneId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneId'; ofType: null };
				};
			};
			shaderVersionId: {
				name: 'shaderVersionId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderVersionId'; ofType: null };
				};
			};
		};
	};
	CaptureId: unknown;
	DateTime: unknown;
	Float: unknown;
	Int: unknown;
	PageInfo: {
		kind: 'OBJECT';
		name: 'PageInfo';
		fields: {
			endCursor: { name: 'endCursor'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			hasNextPage: {
				name: 'hasNextPage';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			hasPreviousPage: {
				name: 'hasPreviousPage';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			startCursor: { name: 'startCursor'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	PresetNode: {
		kind: 'OBJECT';
		name: 'PresetNode';
		fields: {
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ScenePresetId'; ofType: null };
				};
			};
			moonPhase: { name: 'moonPhase'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			sceneId: {
				name: 'sceneId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneId'; ofType: null };
				};
			};
			slug: {
				name: 'slug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			sortOrder: {
				name: 'sortOrder';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			timeOfDayTicks: {
				name: 'timeOfDayTicks';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			updatedAt: {
				name: 'updatedAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			weather: {
				name: 'weather';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			weatherIntensity: {
				name: 'weatherIntensity';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
		};
	};
	QueryRoot: {
		kind: 'OBJECT';
		name: 'QueryRoot';
		fields: {
			scene: { name: 'scene'; type: { kind: 'OBJECT'; name: 'SceneNode'; ofType: null } };
			scenes: {
				name: 'scenes';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'SceneConnection'; ofType: null };
				};
			};
		};
	};
	SceneConnection: {
		kind: 'OBJECT';
		name: 'SceneConnection';
		fields: {
			edges: {
				name: 'edges';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'SceneEdge'; ofType: null };
						};
					};
				};
			};
			pageInfo: {
				name: 'pageInfo';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'PageInfo'; ofType: null };
				};
			};
			totalCount: {
				name: 'totalCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
	SceneEdge: {
		kind: 'OBJECT';
		name: 'SceneEdge';
		fields: {
			cursor: {
				name: 'cursor';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			node: {
				name: 'node';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'SceneNode'; ofType: null };
				};
			};
		};
	};
	SceneId: unknown;
	SceneNode: {
		kind: 'OBJECT';
		name: 'SceneNode';
		fields: {
			active: {
				name: 'active';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			captureCount: {
				name: 'captureCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			dimension: {
				name: 'dimension';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneId'; ofType: null };
				};
			};
			imagePath: { name: 'imagePath'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			presets: {
				name: 'presets';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'PresetNode'; ofType: null };
						};
					};
				};
			};
			slug: {
				name: 'slug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			version: { name: 'version'; type: { kind: 'OBJECT'; name: 'SceneVersionNode'; ofType: null } };
		};
	};
	ScenePresetId: unknown;
	SceneVersionId: unknown;
	SceneVersionNode: {
		kind: 'OBJECT';
		name: 'SceneVersionNode';
		fields: {
			biome: { name: 'biome'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			fov: {
				name: 'fov';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneVersionId'; ofType: null };
				};
			};
			moonPhase: { name: 'moonPhase'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			pitch: {
				name: 'pitch';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
			renderDistance: {
				name: 'renderDistance';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			sceneId: {
				name: 'sceneId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneId'; ofType: null };
				};
			};
			timeOfDayTicks: {
				name: 'timeOfDayTicks';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			weather: {
				name: 'weather';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			weatherIntensity: {
				name: 'weatherIntensity';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
			x: {
				name: 'x';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
			y: {
				name: 'y';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
			yaw: {
				name: 'yaw';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
			z: {
				name: 'z';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Float'; ofType: null };
				};
			};
		};
	};
	ShaderVersionId: unknown;
	String: unknown;
	SubscriptionRoot: {
		kind: 'OBJECT';
		name: 'SubscriptionRoot';
		fields: {
			captureCompleted: {
				name: 'captureCompleted';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'CaptureCompletedEvent'; ofType: null };
				};
			};
		};
	};
};

/** An IntrospectionQuery representation of your schema.
 *
 * @remarks
 * This is an introspection of your schema saved as a file by GraphQLSP.
 * It will automatically be used by `gql.tada` to infer the types of your GraphQL documents.
 * If you need to reuse this data or update your `scalars`, update `tadaOutputLocation` to
 * instead save to a .ts instead of a .d.ts file.
 */
export type introspection = {
	name: never;
	query: 'QueryRoot';
	mutation: never;
	subscription: 'SubscriptionRoot';
	types: introspection_types;
};

import * as gqlTada from 'gql.tada';

declare module 'gql.tada' {
	interface setupSchema {
		introspection: introspection;
	}
}
