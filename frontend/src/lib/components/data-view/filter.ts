/**
 * A typed filter descriptor used by `createListState` to initialize reactive
 * filter state and determine URL serialization/deserialization strategy.
 */
export interface FilterDescriptor<T> {
	default: T | null;
	/** Custom parser for URL string values. When omitted, booleans are parsed
	 *  via `=== 'true'`, numbers via `Number()`, and strings are used as-is. */
	parse?: (raw: string) => T;
}

/** Options bag for `filter()` when custom parsing or a nullable default is needed. */
interface FilterOptions<T> {
	default?: T | null;
	parse?: (raw: string) => T;
}

/**
 * Creates a typed filter descriptor for use in `createListState` config.
 *
 * When no argument is passed (with a generic), default is `null`:
 * ```ts
 * filter<string>()       // { default: null } typed as FilterDescriptor<string>
 * ```
 *
 * When a value is passed, the type is inferred and the value becomes the default:
 * ```ts
 * filter(false)          // { default: false } typed as FilterDescriptor<boolean>
 * filter(25)             // { default: 25 } typed as FilterDescriptor<number>
 * filter('popular')      // { default: 'popular' } typed as FilterDescriptor<string>
 * ```
 *
 * When an options bag is passed, custom parsing can be provided:
 * ```ts
 * filter<number>({ parse: Number })   // nullable number with URL parsing
 * ```
 */
export function filter<T>(): FilterDescriptor<T>;
export function filter<T>(defaultValue: T): FilterDescriptor<T>;
export function filter<T>(opts: FilterOptions<T>): FilterDescriptor<T>;
export function filter<T>(arg?: T | FilterOptions<T>): FilterDescriptor<T> {
	if (arg === undefined) {
		return { default: null };
	}
	if (typeof arg === 'object' && arg !== null && ('parse' in arg || 'default' in arg)) {
		const opts = arg;
		return {
			default: opts.default === undefined ? null : opts.default,
			parse: opts.parse
		};
	}
	return { default: arg as T };
}

/**
 * Extract filter variables from URL search params using a filter config.
 *
 * Returns only non-default values, ready to be passed as GraphQL variables.
 * If `filterVariable` is provided, non-default values are nested under that key
 * (e.g., `{ filters: { shaderSlug: "bsl" } }`). Otherwise they're top-level.
 */
export function extractFiltersFromUrl<F extends Record<string, FilterDescriptor<unknown>>>(
	searchParams: URLSearchParams,
	filterConfig: F,
	filterVariable?: string
): Record<string, unknown> {
	const filterVars: Record<string, unknown> = {};
	for (const key of Object.keys(filterConfig)) {
		const raw = searchParams.get(key);
		if (raw === null) continue;
		const descriptor = filterConfig[key as keyof F];
		filterVars[key] = parseFilterValue(raw, descriptor);
	}

	if (Object.keys(filterVars).length === 0) return {};

	if (filterVariable) {
		return { [filterVariable]: filterVars };
	}
	return filterVars;
}

export function parseFilterValue<T>(raw: string, descriptor: FilterDescriptor<T>): unknown {
	if (descriptor.parse) {
		return descriptor.parse(raw);
	}
	const defaultValue = descriptor.default;
	if (Array.isArray(defaultValue)) {
		return raw ? raw.split(',') : [];
	}
	if (defaultValue === null || typeof defaultValue === 'string') {
		return raw;
	}
	if (typeof defaultValue === 'number') {
		const n = Number(raw);
		return Number.isNaN(n) ? defaultValue : n;
	}
	if (typeof defaultValue === 'boolean') {
		return raw === 'true';
	}
	return raw;
}

/** Check whether a filter value is at its default (should be omitted from URL). */
export function isFilterDefault(val: unknown, def: unknown): boolean {
	if (val === null || val === def) return true;
	if (Array.isArray(val)) {
		if (val.length === 0) return true;
		if (Array.isArray(def) && val.length === def.length && val.every((v, i) => v === def[i]))
			return true;
		return false;
	}
	return false;
}

/** Serialize a filter value for URL params. */
export function serializeFilterValue(val: unknown): string | null {
	if (val == null) return null;
	if (Array.isArray(val)) return val.length > 0 ? val.join(',') : null;
	if (typeof val === 'string') return val;
	if (typeof val === 'number' || typeof val === 'boolean') return val.toString();
	return JSON.stringify(val);
}
