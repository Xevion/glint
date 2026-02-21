/** Relay connection shape returned by GraphQL cursor pagination. */
export interface RelayConnection<TNode> {
	edges: { node: TNode }[];
	pageInfo: {
		hasNextPage: boolean;
		endCursor?: string | null;
	};
	totalCount: number;
}

/** Returns an empty RelayConnection for use as a fallback/error value. */
export function emptyConnection<T>(): RelayConnection<T> {
	return { edges: [], pageInfo: { hasNextPage: false }, totalCount: 0 };
}

/** Unwrap a Relay connection into a flat structure for state management. */
export function unwrapConnection<T>(connection: RelayConnection<T>) {
	return {
		items: connection.edges.map((e) => e.node),
		endCursor: connection.pageInfo.endCursor ?? null,
		hasNextPage: connection.pageInfo.hasNextPage,
		totalCount: connection.totalCount
	};
}

/**
 * Append new items to an existing array, deduplicating by the provided key function.
 */
export function appendDeduplicatedItems<T>(
	existing: T[],
	incoming: T[],
	keyFn: (item: T) => string | number
): T[] {
	const existingKeys = new Set(existing.map(keyFn));
	return [...existing, ...incoming.filter((item) => !existingKeys.has(keyFn(item)))];
}
