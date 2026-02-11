import { type LogRecord, configure, getLogger } from '@logtape/logtape';
import type { Plugin, ViteDevServer } from 'vite';

interface JsonLogEntry {
	timestamp: string;
	level: string;
	message: string;
	target: string;
	[key: string]: unknown;
}

function jsonFormatter(record: LogRecord): string {
	const entry: JsonLogEntry = {
		timestamp: new Date().toISOString(),
		level: record.level.toLowerCase(),
		message: record.message.join(' '),
		target: 'vite'
	};

	if (record.properties && Object.keys(record.properties).length > 0) {
		Object.assign(entry, record.properties);
	}

	return JSON.stringify(entry) + '\n';
}

function stripAnsi(str: string): string {
	// eslint-disable-next-line no-control-regex
	return str.replace(/\u001b\[[0-9;]*m/g, '').trim();
}

let loggerConfigured = false;

export function jsonLogger(): Plugin {
	const useJsonLogs = process.env.LOG_JSON === 'true' || process.env.LOG_JSON === '1';

	if (!useJsonLogs) {
		return {
			name: 'vite-plugin-json-logger'
		};
	}

	const configureLogger = async () => {
		if (loggerConfigured) return;
		await configure({
			sinks: {
				json: (record: LogRecord) => {
					process.stdout.write(jsonFormatter(record));
				}
			},
			filters: {},
			loggers: [
				{
					category: ['logtape', 'meta'],
					lowestLevel: 'warning',
					sinks: ['json']
				},
				{
					category: [],
					lowestLevel: 'debug',
					sinks: ['json']
				}
			]
		});
		loggerConfigured = true;
	};

	let server: ViteDevServer;
	const ignoredMessages = new Set(['press h + enter to show help', 'ready in']);

	return {
		name: 'vite-plugin-json-logger',

		async config() {
			await configureLogger();
			const logger = getLogger(['vite']);

			return {
				customLogger: {
					info(msg: string) {
						const cleaned = stripAnsi(msg);
						if (!cleaned || ignoredMessages.has(cleaned) || cleaned.includes('VITE v')) {
							return;
						}
						logger.info(cleaned);
					},
					warn(msg: string) {
						const cleaned = stripAnsi(msg);
						if (cleaned) {
							logger.warn(cleaned);
						}
					},
					error(msg: string) {
						const cleaned = stripAnsi(msg);
						if (cleaned) {
							logger.error(cleaned);
						}
					},
					clearScreen() {
						// Intentionally empty — JSON mode doesn't clear the terminal
					},
					hasErrorLogged() {
						return false;
					},
					hasWarned: false,
					warnOnce(msg: string) {
						this.warn(msg);
					}
				}
			};
		},

		configureServer(s) {
			server = s;
			const logger = getLogger(['vite']);

			server.printUrls = () => {
				const urls = server.resolvedUrls;
				if (urls) {
					logger.info('dev server running', {
						local: urls.local,
						network: urls.network
					});
				}
			};

			server.httpServer?.once('listening', () => {
				logger.info('server listening');
			});

			server.ws.on('connection', () => {
				logger.info('client connected');
			});
		},

		handleHotUpdate({ file, modules }) {
			const logger = getLogger(['vite']);
			logger.info('hmr update', {
				file: file.replace(process.cwd(), ''),
				modules: modules.length
			});
			return modules;
		},

		buildStart() {
			const logger = getLogger(['vite']);
			logger.info('build started');
		},

		buildEnd() {
			const logger = getLogger(['vite']);
			logger.info('build ended');
		}
	};
}
