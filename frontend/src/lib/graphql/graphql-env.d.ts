/* eslint-disable */
/* prettier-ignore */

export type introspection_types = {
	AdminShaderList: {
		kind: 'OBJECT';
		name: 'AdminShaderList';
		fields: {
			items: {
				name: 'items';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null };
						};
					};
				};
			};
			page: {
				name: 'page';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			pageSize: {
				name: 'pageSize';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			total: {
				name: 'total';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
	BackgroundId: unknown;
	BackgroundNode: {
		kind: 'OBJECT';
		name: 'BackgroundNode';
		fields: {
			contentType: { name: 'contentType'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			enabled: {
				name: 'enabled';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			fileSizeBytes: { name: 'fileSizeBytes'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			height: { name: 'height'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'BackgroundId'; ofType: null };
				};
			};
			imagePath: {
				name: 'imagePath';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			originalFilename: {
				name: 'originalFilename';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			sortOrder: {
				name: 'sortOrder';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			themeMode: {
				name: 'themeMode';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'ThemeModeEnum'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			updatedAt: {
				name: 'updatedAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			width: { name: 'width'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
		};
	};
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
	CaptureConnection: {
		kind: 'OBJECT';
		name: 'CaptureConnection';
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
							ofType: { kind: 'OBJECT'; name: 'CaptureEdge'; ofType: null };
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
	CaptureEdge: {
		kind: 'OBJECT';
		name: 'CaptureEdge';
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
					ofType: { kind: 'OBJECT'; name: 'CaptureNode'; ofType: null };
				};
			};
		};
	};
	CaptureFreshnessEnum: {
		name: 'CaptureFreshnessEnum';
		enumValues: 'FRESH' | 'STALE' | 'SUPERSEDED';
	};
	CaptureId: unknown;
	CaptureNode: {
		kind: 'OBJECT';
		name: 'CaptureNode';
		fields: {
			capturedAt: { name: 'capturedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'CaptureId'; ofType: null };
				};
			};
			imagePath: {
				name: 'imagePath';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			profileId: {
				name: 'profileId';
				type: { kind: 'SCALAR'; name: 'ShaderVersionProfileId'; ofType: null };
			};
			resolutionHeight: {
				name: 'resolutionHeight';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			resolutionWidth: {
				name: 'resolutionWidth';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
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
			status: {
				name: 'status';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'CaptureStatusEnum'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	CaptureRunId: unknown;
	CaptureRunItemNode: {
		kind: 'OBJECT';
		name: 'CaptureRunItemNode';
		fields: {
			captureId: { name: 'captureId'; type: { kind: 'SCALAR'; name: 'CaptureId'; ofType: null } };
			completedAt: { name: 'completedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			durationMs: { name: 'durationMs'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			errorLog: { name: 'errorLog'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			errorMessage: { name: 'errorMessage'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			imagePath: { name: 'imagePath'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			presetId: { name: 'presetId'; type: { kind: 'SCALAR'; name: 'ScenePresetId'; ofType: null } };
			profileDisplayName: {
				name: 'profileDisplayName';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			profileId: {
				name: 'profileId';
				type: { kind: 'SCALAR'; name: 'ShaderVersionProfileId'; ofType: null };
			};
			profileName: { name: 'profileName'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			runId: {
				name: 'runId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'CaptureRunId'; ofType: null };
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
			sceneName: {
				name: 'sceneName';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderName: {
				name: 'shaderName';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderSlug: {
				name: 'shaderSlug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderVersion: {
				name: 'shaderVersion';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
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
			startedAt: { name: 'startedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			status: {
				name: 'status';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'CaptureRunItemStatusEnum'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	CaptureRunItemStatusEnum: {
		name: 'CaptureRunItemStatusEnum';
		enumValues: 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'SKIPPED';
	};
	CaptureRunNode: {
		kind: 'OBJECT';
		name: 'CaptureRunNode';
		fields: {
			agentId: { name: 'agentId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			completedAt: { name: 'completedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			completedItems: {
				name: 'completedItems';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			failedItems: {
				name: 'failedItems';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'CaptureRunId'; ofType: null };
				};
			};
			imageFormat: {
				name: 'imageFormat';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			items: {
				name: 'items';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'CaptureRunItemNode'; ofType: null };
						};
					};
				};
			};
			resolutionHeight: {
				name: 'resolutionHeight';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			resolutionWidth: {
				name: 'resolutionWidth';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			skippedItems: {
				name: 'skippedItems';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			startedAt: {
				name: 'startedAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			status: {
				name: 'status';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'CaptureRunStatusEnum'; ofType: null };
				};
			};
			totalItems: {
				name: 'totalItems';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
	CaptureRunStatusEnum: {
		name: 'CaptureRunStatusEnum';
		enumValues: 'RUNNING' | 'COMPLETED' | 'PARTIAL' | 'FAILED' | 'TIMED_OUT';
	};
	CaptureStatusEnum: { name: 'CaptureStatusEnum'; enumValues: 'UPLOADING' | 'COMPLETED' | 'FAILED' };
	CaptureWithContextNode: {
		kind: 'OBJECT';
		name: 'CaptureWithContextNode';
		fields: {
			capturedAt: { name: 'capturedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			fileSizeBytes: { name: 'fileSizeBytes'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			freshness: {
				name: 'freshness';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'CaptureFreshnessEnum'; ofType: null };
				};
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'CaptureId'; ofType: null };
				};
			};
			imagePath: {
				name: 'imagePath';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			presetId: { name: 'presetId'; type: { kind: 'SCALAR'; name: 'ScenePresetId'; ofType: null } };
			presetName: { name: 'presetName'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			presetSlug: { name: 'presetSlug'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			profileDisplayName: {
				name: 'profileDisplayName';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			profileId: {
				name: 'profileId';
				type: { kind: 'SCALAR'; name: 'ShaderVersionProfileId'; ofType: null };
			};
			profileName: { name: 'profileName'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			resolutionHeight: {
				name: 'resolutionHeight';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			resolutionWidth: {
				name: 'resolutionWidth';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			runId: { name: 'runId'; type: { kind: 'SCALAR'; name: 'CaptureRunId'; ofType: null } };
			runStatus: { name: 'runStatus'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			sceneId: {
				name: 'sceneId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'SceneId'; ofType: null };
				};
			};
			sceneName: { name: 'sceneName'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			sceneSlug: { name: 'sceneSlug'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			shaderAuthor: { name: 'shaderAuthor'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			shaderName: {
				name: 'shaderName';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderSlug: {
				name: 'shaderSlug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderVersion: {
				name: 'shaderVersion';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	CategoryNode: {
		kind: 'OBJECT';
		name: 'CategoryNode';
		fields: {
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			id: {
				name: 'id';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
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
		};
	};
	CreateShaderVersionInput: {
		kind: 'INPUT_OBJECT';
		name: 'CreateShaderVersionInput';
		isOneOf: false;
		inputFields: [
			{
				name: 'version';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
				defaultValue: null;
			},
			{
				name: 'modrinthVersionId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'downloadUrl';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{ name: 'fileHash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null }; defaultValue: null }
		];
	};
	DateTime: unknown;
	ExtractionStatusEnum: {
		name: 'ExtractionStatusEnum';
		enumValues: 'PENDING' | 'COMPLETED' | 'FAILED' | 'SKIPPED';
	};
	ExtractionSummaryNode: {
		kind: 'OBJECT';
		name: 'ExtractionSummaryNode';
		fields: {
			completed: {
				name: 'completed';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			failed: {
				name: 'failed';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			pending: {
				name: 'pending';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			skipped: {
				name: 'skipped';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			total: {
				name: 'total';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
	FeatureNode: {
		kind: 'OBJECT';
		name: 'FeatureNode';
		fields: {
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			id: {
				name: 'id';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
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
		};
	};
	FeaturedPairNode: {
		kind: 'OBJECT';
		name: 'FeaturedPairNode';
		fields: {
			left: {
				name: 'left';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'FeaturedSideNode'; ofType: null };
				};
			};
			right: {
				name: 'right';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'FeaturedSideNode'; ofType: null };
				};
			};
		};
	};
	FeaturedSideNode: {
		kind: 'OBJECT';
		name: 'FeaturedSideNode';
		fields: {
			imagePath: {
				name: 'imagePath';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			sceneName: {
				name: 'sceneName';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderAuthor: { name: 'shaderAuthor'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			shaderName: {
				name: 'shaderName';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderSlug: {
				name: 'shaderSlug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderVersion: {
				name: 'shaderVersion';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	Float: unknown;
	ID: unknown;
	Int: unknown;
	JSON: unknown;
	MutationRoot: {
		kind: 'OBJECT';
		name: 'MutationRoot';
		fields: {
			createShaderVersion: {
				name: 'createShaderVersion';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderVersionNode'; ofType: null };
				};
			};
			deleteShader: {
				name: 'deleteShader';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			linkShaderPlatform: {
				name: 'linkShaderPlatform';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null };
				};
			};
			syncShader: {
				name: 'syncShader';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null };
				};
			};
			updateShader: {
				name: 'updateShader';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null };
				};
			};
		};
	};
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
			adminCaptureRun: {
				name: 'adminCaptureRun';
				type: { kind: 'OBJECT'; name: 'CaptureRunNode'; ofType: null };
			};
			adminShader: { name: 'adminShader'; type: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null } };
			adminShaders: {
				name: 'adminShaders';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'AdminShaderList'; ofType: null };
				};
			};
			allBackgrounds: {
				name: 'allBackgrounds';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'BackgroundNode'; ofType: null };
						};
					};
				};
			};
			backgrounds: {
				name: 'backgrounds';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'BackgroundNode'; ofType: null };
						};
					};
				};
			};
			captures: {
				name: 'captures';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'CaptureConnection'; ofType: null };
				};
			};
			featured: {
				name: 'featured';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'FeaturedPairNode'; ofType: null };
						};
					};
				};
			};
			scene: { name: 'scene'; type: { kind: 'OBJECT'; name: 'SceneNode'; ofType: null } };
			scenes: {
				name: 'scenes';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'SceneConnection'; ofType: null };
				};
			};
			shader: { name: 'shader'; type: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null } };
			shaders: {
				name: 'shaders';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderConnection'; ofType: null };
				};
			};
			stats: {
				name: 'stats';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'StatsNode'; ofType: null };
				};
			};
			trendingShaders: {
				name: 'trendingShaders';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'TrendingShaderNode'; ofType: null };
						};
					};
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
			captures: {
				name: 'captures';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'CaptureWithContextNode'; ofType: null };
						};
					};
				};
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
	ShaderAuthorNode: {
		kind: 'OBJECT';
		name: 'ShaderAuthorNode';
		fields: {
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			platform: {
				name: 'platform';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			shaderId: {
				name: 'shaderId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderId'; ofType: null };
				};
			};
			url: { name: 'url'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	ShaderConnection: {
		kind: 'OBJECT';
		name: 'ShaderConnection';
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
							ofType: { kind: 'OBJECT'; name: 'ShaderEdge'; ofType: null };
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
	ShaderEdge: {
		kind: 'OBJECT';
		name: 'ShaderEdge';
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
					ofType: { kind: 'OBJECT'; name: 'ShaderNode'; ofType: null };
				};
			};
		};
	};
	ShaderId: unknown;
	ShaderNode: {
		kind: 'OBJECT';
		name: 'ShaderNode';
		fields: {
			authors: {
				name: 'authors';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'ShaderAuthorNode'; ofType: null };
						};
					};
				};
			};
			captureEnabled: {
				name: 'captureEnabled';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			captures: {
				name: 'captures';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'CaptureWithContextNode'; ofType: null };
						};
					};
				};
			};
			categories: {
				name: 'categories';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'CategoryNode'; ofType: null };
						};
					};
				};
			};
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			curseforgeId: { name: 'curseforgeId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			extractionSummary: {
				name: 'extractionSummary';
				type: { kind: 'OBJECT'; name: 'ExtractionSummaryNode'; ofType: null };
			};
			features: {
				name: 'features';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'FeatureNode'; ofType: null };
						};
					};
				};
			};
			iconUrl: { name: 'iconUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderId'; ofType: null };
				};
			};
			imagePath: { name: 'imagePath'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			lastSyncedAt: { name: 'lastSyncedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			latestVersion: { name: 'latestVersion'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			licenseId: { name: 'licenseId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			metadata: {
				name: 'metadata';
				type: { kind: 'OBJECT'; name: 'ShaderVersionMetadataNode'; ofType: null };
			};
			modrinthId: { name: 'modrinthId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			preferredVersionId: {
				name: 'preferredVersionId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			profiles: {
				name: 'profiles';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'ShaderVersionProfileNode'; ofType: null };
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
			sourceUrl: { name: 'sourceUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			updatedAt: {
				name: 'updatedAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			upstreamDownloads: {
				name: 'upstreamDownloads';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			upstreamUpdatedAt: {
				name: 'upstreamUpdatedAt';
				type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
			};
			versionCount: {
				name: 'versionCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			versions: {
				name: 'versions';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: {
						kind: 'LIST';
						name: never;
						ofType: {
							kind: 'NON_NULL';
							name: never;
							ofType: { kind: 'OBJECT'; name: 'ShaderVersionDetailNode'; ofType: null };
						};
					};
				};
			};
			viewCount: {
				name: 'viewCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			websiteUrl: { name: 'websiteUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	ShaderVersionDetailNode: {
		kind: 'OBJECT';
		name: 'ShaderVersionDetailNode';
		fields: {
			captureCount: {
				name: 'captureCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			version: {
				name: 'version';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'OBJECT'; name: 'ShaderVersionNode'; ofType: null };
				};
			};
		};
	};
	ShaderVersionId: unknown;
	ShaderVersionMetadataNode: {
		kind: 'OBJECT';
		name: 'ShaderVersionMetadataNode';
		fields: {
			dimensionSupport: {
				name: 'dimensionSupport';
				type: { kind: 'SCALAR'; name: 'JSON'; ofType: null };
			};
			extractedAt: {
				name: 'extractedAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			filePaths: { name: 'filePaths'; type: { kind: 'SCALAR'; name: 'JSON'; ofType: null } };
			hasCustomTextures: {
				name: 'hasCustomTextures';
				type: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
			};
			irisFeaturesOptional: {
				name: 'irisFeaturesOptional';
				type: { kind: 'SCALAR'; name: 'JSON'; ofType: null };
			};
			irisFeaturesRequired: {
				name: 'irisFeaturesRequired';
				type: { kind: 'SCALAR'; name: 'JSON'; ofType: null };
			};
			pipelineFeatures: {
				name: 'pipelineFeatures';
				type: { kind: 'SCALAR'; name: 'JSON'; ofType: null };
			};
			settingsScreen: { name: 'settingsScreen'; type: { kind: 'SCALAR'; name: 'JSON'; ofType: null } };
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
	ShaderVersionNode: {
		kind: 'OBJECT';
		name: 'ShaderVersionNode';
		fields: {
			captureFailureCount: {
				name: 'captureFailureCount';
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
			curseforgeFileId: {
				name: 'curseforgeFileId';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			downloadUrl: { name: 'downloadUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			extractedAt: { name: 'extractedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			extractionError: {
				name: 'extractionError';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			extractionStatus: {
				name: 'extractionStatus';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'ENUM'; name: 'ExtractionStatusEnum'; ofType: null };
				};
			};
			fileHash: { name: 'fileHash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			fileSize: { name: 'fileSize'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			gameVersions: {
				name: 'gameVersions';
				type: {
					kind: 'LIST';
					name: never;
					ofType: {
						kind: 'NON_NULL';
						name: never;
						ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
					};
				};
			};
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderVersionId'; ofType: null };
				};
			};
			lastCaptureError: {
				name: 'lastCaptureError';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			modrinthVersionId: {
				name: 'modrinthVersionId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			releaseChannel: {
				name: 'releaseChannel';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			shaderId: {
				name: 'shaderId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderId'; ofType: null };
				};
			};
			upstreamPublishedAt: {
				name: 'upstreamPublishedAt';
				type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
			};
			version: {
				name: 'version';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
		};
	};
	ShaderVersionProfileId: unknown;
	ShaderVersionProfileNode: {
		kind: 'OBJECT';
		name: 'ShaderVersionProfileNode';
		fields: {
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			displayName: {
				name: 'displayName';
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
					ofType: { kind: 'SCALAR'; name: 'ShaderVersionProfileId'; ofType: null };
				};
			};
			label: { name: 'label'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			options: { name: 'options'; type: { kind: 'SCALAR'; name: 'JSON'; ofType: null } };
			shaderVersionId: {
				name: 'shaderVersionId';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderVersionId'; ofType: null };
				};
			};
			sortOrder: {
				name: 'sortOrder';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
	StatsNode: {
		kind: 'OBJECT';
		name: 'StatsNode';
		fields: {
			captureCount: {
				name: 'captureCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			latestCaptureAt: {
				name: 'latestCaptureAt';
				type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
			};
			sceneCount: {
				name: 'sceneCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			shaderCount: {
				name: 'shaderCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
		};
	};
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
	ThemeModeEnum: { name: 'ThemeModeEnum'; enumValues: 'LIGHT' | 'DARK' | 'BOTH' };
	TrendingShaderNode: {
		kind: 'OBJECT';
		name: 'TrendingShaderNode';
		fields: {
			captureEnabled: {
				name: 'captureEnabled';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				};
			};
			createdAt: {
				name: 'createdAt';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
				};
			};
			curseforgeId: { name: 'curseforgeId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			description: { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			iconUrl: { name: 'iconUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			id: {
				name: 'id';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'ShaderId'; ofType: null };
				};
			};
			imagePath: { name: 'imagePath'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			lastSyncedAt: { name: 'lastSyncedAt'; type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null } };
			licenseId: { name: 'licenseId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			modrinthId: { name: 'modrinthId'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			name: {
				name: 'name';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			preferredVersionId: {
				name: 'preferredVersionId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
			};
			slug: {
				name: 'slug';
				type: {
					kind: 'NON_NULL';
					name: never;
					ofType: { kind: 'SCALAR'; name: 'String'; ofType: null };
				};
			};
			sourceUrl: { name: 'sourceUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			thumbhash: { name: 'thumbhash'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
			trendingViews: {
				name: 'trendingViews';
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
			upstreamDownloads: {
				name: 'upstreamDownloads';
				type: { kind: 'SCALAR'; name: 'Int'; ofType: null };
			};
			upstreamUpdatedAt: {
				name: 'upstreamUpdatedAt';
				type: { kind: 'SCALAR'; name: 'DateTime'; ofType: null };
			};
			viewCount: {
				name: 'viewCount';
				type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null } };
			};
			websiteUrl: { name: 'websiteUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null } };
		};
	};
	UpdateShaderInput: {
		kind: 'INPUT_OBJECT';
		name: 'UpdateShaderInput';
		isOneOf: false;
		inputFields: [
			{ name: 'name'; type: { kind: 'SCALAR'; name: 'String'; ofType: null }; defaultValue: null },
			{
				name: 'description';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'modrinthId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'curseforgeId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'websiteUrl';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'captureEnabled';
				type: { kind: 'SCALAR'; name: 'Boolean'; ofType: null };
				defaultValue: null;
			},
			{
				name: 'preferredVersionId';
				type: { kind: 'SCALAR'; name: 'String'; ofType: null };
				defaultValue: null;
			}
		];
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
	mutation: 'MutationRoot';
	subscription: 'SubscriptionRoot';
	types: introspection_types;
};

import * as gqlTada from 'gql.tada';

declare module 'gql.tada' {
	interface setupSchema {
		introspection: introspection;
	}
}
