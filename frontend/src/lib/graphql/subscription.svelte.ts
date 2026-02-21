import { browser } from '$app/environment';
import type { TypedDocumentNode } from '@graphql-typed-document-node/core';
import { print } from 'graphql';
import { type Client as WSClient, createClient as createWSClient } from 'graphql-ws';

let wsClient: WSClient | null = null;

function getOrCreateWSClient(): WSClient {
	if (!browser) throw new Error('WebSocket client is browser-only');

	if (!wsClient) {
		const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
		wsClient = createWSClient({
			url: `${protocol}//${location.host}/api/graphql/ws`,
			lazy: true,
			retryAttempts: Infinity,
			retryWait: async (retries) => {
				// Exponential backoff: 1s, 2s, 4s, 8s, capped at 30s
				const delay = Math.min(1000 * 2 ** retries, 30_000);
				await new Promise((resolve) => setTimeout(resolve, delay));
			}
		});
	}
	return wsClient;
}

/** Dispose the shared WS client. Useful for teardown in tests or full-page cleanup. */
export async function disposeWSClient(): Promise<void> {
	await wsClient?.dispose();
	wsClient = null;
}

/**
 * Subscribe to a GraphQL subscription with automatic lifecycle management.
 *
 * Starts the subscription immediately (browser-only) and stops it when the
 * enclosing reactive scope is destroyed (component unmount, effect cleanup).
 * Use inside a component script or `$effect` — the cleanup is automatic.
 */
export function useSubscription<Data, Variables extends Record<string, unknown>>(
	document: TypedDocumentNode<Data, Variables>,
	variables: Variables
) {
	let data = $state<Data | null>(null);
	let error = $state<Error | null>(null);

	let unsubscribe: (() => void) | null = null;

	function start() {
		if (!browser || unsubscribe) return;

		unsubscribe = getOrCreateWSClient().subscribe(
			{ query: print(document), variables },
			{
				next: (result) => {
					if (result.data) {
						data = result.data as Data;
					}
					error = null;
				},
				error: (err) => {
					error = err instanceof Error ? err : new Error(String(err));
				},
				complete: () => {
					unsubscribe = null;
				}
			}
		);
	}

	function stop() {
		unsubscribe?.();
		unsubscribe = null;
	}

	// Auto-lifecycle: start on creation, stop when reactive scope is destroyed
	$effect(() => {
		start();
		return () => stop();
	});

	return {
		get data() {
			return data;
		},
		get error() {
			return error;
		}
	};
}
