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
 * Append new items to an existing array, deduplicating by `id` when present.
 * Items without an `id` property are always appended.
 */
export function appendDeduplicatedItems<T>(existing: T[], incoming: T[]): T[] {
	const getId = (item: T): string | number | undefined => {
		const rec = item as Record<string, unknown>;
		const id = rec.id;
		return typeof id === 'string' || typeof id === 'number' ? id : undefined;
	};
	const existingIds = new Set(existing.map(getId).filter((id) => id != null));
	const uniqueNew = incoming.filter((item) => {
		const id = getId(item);
		return id == null || !existingIds.has(id);
	});
	return [...existing, ...uniqueNew];
}
