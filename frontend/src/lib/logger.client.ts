import { dev } from '$app/environment';
import { configureSync, getConsoleSink } from '@logtape/logtape';

let initialized = false;

/**
 * Initialize LogTape for the browser environment.
 * Uses configureSync since top-level await isn't available in client hooks.
 */
export function initClientLogger(): void {
	if (initialized) return;
	initialized = true;

	configureSync({
		sinks: {
			console: getConsoleSink()
		},
		loggers: [
			{
				category: ['logtape', 'meta'],
				lowestLevel: 'warning',
				sinks: ['console']
			},
			{
				category: [],
				lowestLevel: dev ? 'debug' : 'warning',
				sinks: ['console']
			}
		]
	});
}
