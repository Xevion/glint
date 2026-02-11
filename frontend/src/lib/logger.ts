import { dev } from '$app/environment';
import { configure, getConsoleSink, type LogRecord } from '@logtape/logtape';
import { requestContext } from '$lib/server/context';

interface JsonLogEntry {
	timestamp: string;
	level: string;
	message: string;
	target: string;
	[key: string]: unknown;
}

function jsonFormatter(record: LogRecord): string {
	const ctx = requestContext.getStore();
	const categoryTarget = record.category.join(':');
	const entry: JsonLogEntry = {
		timestamp: new Date().toISOString(),
		level: record.level.toLowerCase(),
		message: record.message.join(' '),
		target: categoryTarget ? `bun:${categoryTarget}` : 'bun',
		...(ctx?.requestId && { req_id: ctx.requestId })
	};

	if (record.properties && Object.keys(record.properties).length > 0) {
		Object.assign(entry, record.properties);
	}

	return JSON.stringify(entry) + '\n';
}

/**
 * Determine whether JSON logging should be used.
 *
 * Priority:
 * 1. LOG_JSON env var explicitly set → use that value
 * 2. Otherwise → JSON in production, console in development
 */
function shouldUseJson(): boolean {
	const explicit = process.env.LOG_JSON;
	if (explicit !== undefined) {
		return explicit === 'true' || explicit === '1';
	}
	return !dev;
}

/** Normalize a log level string to logtape's expected values.
 *  Accepts both tracing-style ('warn') and logtape-style ('warning'). */
function normalizeLevel(raw: string): 'debug' | 'info' | 'warning' | 'error' {
	const level = raw.toLowerCase();
	if (level === 'warn') return 'warning';
	if (['debug', 'info', 'warning', 'error'].includes(level)) {
		return level as 'debug' | 'info' | 'warning' | 'error';
	}
	return dev ? 'debug' : 'info';
}

export async function initLogger() {
	const useJsonLogs = shouldUseJson();

	const logLevel = normalizeLevel(process.env.LOG_LEVEL ?? (dev ? 'debug' : 'info'));

	const jsonSink = (record: LogRecord) => {
		process.stdout.write(jsonFormatter(record));
	};
	const consoleSink = getConsoleSink();

	try {
		await configure({
			sinks: {
				json: useJsonLogs ? jsonSink : consoleSink,
				console: useJsonLogs ? jsonSink : consoleSink
			},
			filters: {},
			loggers: [
				{
					category: ['logtape', 'meta'],
					lowestLevel: 'warning',
					sinks: [useJsonLogs ? 'json' : 'console']
				},
				{
					category: [],
					lowestLevel: logLevel,
					sinks: [useJsonLogs ? 'json' : 'console']
				}
			]
		});
	} catch (error) {
		if (error instanceof Error && error.message.includes('Already configured')) {
			return;
		}
		throw error;
	}
}
