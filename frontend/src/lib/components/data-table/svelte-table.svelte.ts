/**
 * Low-level Svelte 5 bridge for TanStack Table.
 *
 * Adapted from shadcn-svelte's data-table implementation.
 * Most consumers should use `createDataTable` instead of this directly.
 */

import {
	type RowData,
	type TableOptions,
	type TableOptionsResolved,
	type TableState,
	type Updater,
	createTable
} from '@tanstack/table-core';

export function createSvelteTable<TData extends RowData>(options: TableOptions<TData>) {
	const resolvedOptions: TableOptionsResolved<TData> = mergeObjects(
		{
			state: {},
			onStateChange: () => undefined,
			renderFallbackValue: null,
			mergeOptions: (defaultOptions: TableOptions<TData>, opts: Partial<TableOptions<TData>>) => {
				return mergeObjects(defaultOptions, opts);
			}
		},
		options
	);

	const table = createTable(resolvedOptions);
	let state = $state<TableState>(table.initialState);

	function updateOptions() {
		table.setOptions(() => {
			return mergeObjects(resolvedOptions, options, {
				state: mergeObjects(state, options.state ?? {}),
				onStateChange: (updater: Updater<TableState>) => {
					if (typeof updater === 'function') state = updater(state);
					else state = mergeObjects(state, updater);
					options.onStateChange?.(updater);
				}
			});
		});
	}

	updateOptions();

	$effect.pre(() => {
		updateOptions();
	});

	return table;
}

// -- Proxy-based object merging (preserves getter semantics) --

type MaybeThunk<T extends object> = T | (() => T | null | undefined);
type Intersection<T extends readonly unknown[]> = (T extends [infer H, ...infer R]
	? H & Intersection<R>
	: unknown) & {};

/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument */

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function mergeObjects<Sources extends readonly MaybeThunk<any>[]>(
	...sources: Sources
): Intersection<{ [K in keyof Sources]: Sources[K] }> {
	const resolve = <T extends object>(src: MaybeThunk<T>): T | undefined =>
		typeof src === 'function' ? (src() ?? undefined) : src;

	const findSourceWithKey = (key: PropertyKey) => {
		for (let i = sources.length - 1; i >= 0; i--) {
			const obj = resolve(sources[i]);
			if (obj && key in obj) return obj;
		}
		return undefined;
	};

	return new Proxy(Object.create(null), {
		get(_, key) {
			const src = findSourceWithKey(key);
			return src?.[key as never];
		},
		has(_, key) {
			return !!findSourceWithKey(key);
		},
		ownKeys(): (string | symbol)[] {
			// eslint-disable-next-line svelte/prefer-svelte-reactivity
			const all = new Set<string | symbol>();
			for (const s of sources) {
				const obj = resolve(s);
				if (obj) {
					for (const k of Reflect.ownKeys(obj)) {
						all.add(k);
					}
				}
			}
			return [...all];
		},
		getOwnPropertyDescriptor(_, key) {
			const src = findSourceWithKey(key);
			if (!src) return undefined;
			return {
				configurable: true,
				enumerable: true,
				value: (src as Record<PropertyKey, unknown>)[key],
				writable: true
			};
		}
	}) as Intersection<{ [K in keyof Sources]: Sources[K] }>;
}

/* eslint-enable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument */
