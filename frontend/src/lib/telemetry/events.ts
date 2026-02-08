/** Page view (PostHog standard — auto-captured via history mode, typed for manual use) */
export interface PageViewEvent {
	name: '$pageview';
	properties: {
		route: string;
		referrer?: string;
	};
}

/** Shader browsing and interaction events */
export interface ShaderInteractionEvent {
	name: 'shader_interaction';
	properties: {
		action: 'card_click' | 'detail_view' | 'favorite';
		shaderId: string;
		shaderName: string;
	};
}

/** Comparison tool interactions */
export interface ComparisonInteractionEvent {
	name: 'comparison_interaction';
	properties: {
		action: 'add_shader' | 'remove_shader' | 'select_scene' | 'open_comparison' | 'close_comparison';
		shaderId?: string;
		sceneId?: string;
		shaderCount?: number;
	};
}

/** Search and filter interactions */
export interface SearchInteractionEvent {
	name: 'search_interaction';
	properties: {
		action: 'query' | 'filter' | 'sort' | 'clear';
		query?: string;
		filterType?: string;
		filterValue?: string;
		sortBy?: string;
	};
}

/** Error tracking event */
export interface ErrorEvent {
	name: 'error';
	properties: {
		errorType: 'network_error' | 'validation_error' | 'runtime_error' | (string & {});
		message: string;
		stack?: string;
		context?: Record<string, unknown>;
	};
}

/** Discriminated union of all telemetry events */
export type TelemetryEvent =
	| PageViewEvent
	| ShaderInteractionEvent
	| ComparisonInteractionEvent
	| SearchInteractionEvent
	| ErrorEvent;

/** Extract the properties type for a given event name */
export type EventProperties<T extends TelemetryEvent['name']> = Extract<
	TelemetryEvent,
	{ name: T }
>['properties'];
