import type { Component, ComponentProps, Snippet } from 'svelte';

/**
 * Wrapper that lets FlexRender identify a Svelte component returned from a
 * column definition's `cell` or `header` function.
 */
export class RenderComponentConfig<TComponent extends Component> {
	component: TComponent;
	props: ComponentProps<TComponent> | Record<string, never>;
	constructor(
		component: TComponent,
		props: ComponentProps<TComponent> | Record<string, never> = {}
	) {
		this.component = component;
		this.props = props;
	}
}

/**
 * Wrapper that lets FlexRender identify a Svelte snippet returned from a
 * column definition's `cell` or `header` function.
 */
export class RenderSnippetConfig<TProps> {
	snippet: Snippet<[TProps]>;
	params: TProps;
	constructor(snippet: Snippet<[TProps]>, params: TProps) {
		this.snippet = snippet;
		this.params = params;
	}
}

/**
 * Render a Svelte component inside a TanStack Table column definition.
 *
 * @example
 * ```ts
 * {
 *   accessorKey: 'name',
 *   cell: ({ row }) => renderComponent(ShaderNameCell, { shader: row.original }),
 * }
 * ```
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function renderComponent<T extends Component<any>, Props extends ComponentProps<T>>(
	component: T,
	props: Props
) {
	return new RenderComponentConfig(component, props);
}

/**
 * Render a Svelte snippet inside a TanStack Table column definition.
 *
 * @example
 * ```ts
 * {
 *   accessorKey: 'status',
 *   cell: ({ row }) => renderSnippet(statusSnippet, { status: row.original.status }),
 * }
 * ```
 */
export function renderSnippet<TProps>(snippet: Snippet<[TProps]>, params: TProps) {
	return new RenderSnippetConfig(snippet, params);
}
