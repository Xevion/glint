import { browser, dev } from '$app/environment';
import { afterNavigate } from '$app/navigation';
import { env } from '$env/dynamic/public';
import posthog from 'posthog-js';
import type { TelemetryEvent } from './events';

class TelemetryClient {
	private enabled = false;

	/** Initialize PostHog. Must be called once at app startup. */
	init(): void {
		if (!browser) return;

		const key = env.PUBLIC_POSTHOG_KEY;
		const host = env.PUBLIC_POSTHOG_HOST;

		if (!key || !host) {
			this.enabled = false;
			this.log('Telemetry disabled: missing PUBLIC_POSTHOG_KEY or PUBLIC_POSTHOG_HOST');
			return;
		}

		posthog.init(key, {
			api_host: host,
			ui_host: 'https://us.posthog.com',
			defaults: '2025-11-30',
			// Disable PostHog's history monkey-patching — it conflicts with
			// SvelteKit's router. Page views are captured via afterNavigate below.
			capture_pageview: false,
			capture_pageleave: true,
			autocapture: false,
			persistence: 'localStorage'
		});

		if (dev) {
			posthog.debug();
		}

		this.enabled = true;
		this.log('Telemetry initialized');
	}

	/** Wire up SvelteKit-native page view tracking. Call from root layout. */
	trackPageViews(): void {
		afterNavigate(() => {
			if (this.enabled) {
				posthog.capture('$pageview');
			}
		});
	}

	/** Capture a type-safe telemetry event. */
	track<E extends TelemetryEvent>(event: E): void {
		this.log(`track: ${event.name}`, event.properties);
		if (this.enabled) {
			posthog.capture(event.name, event.properties);
		}
	}

	/** Identify the current user. */
	identify(userId: string, properties?: Record<string, unknown>): void {
		this.log('identify', { userId, properties });
		if (this.enabled) {
			posthog.identify(userId, properties);
		}
	}

	/** Reset user identification (e.g. on logout). */
	reset(): void {
		this.log('reset');
		if (this.enabled) {
			posthog.reset();
		}
	}

	/** Convenience method for tracking errors. */
	trackError(
		errorType: string,
		message: string,
		options?: { stack?: string; context?: Record<string, unknown> }
	): void {
		this.track({
			name: 'error',
			properties: {
				errorType,
				message,
				...options
			}
		});
	}

	/** Whether telemetry is actively sending events. */
	isEnabled(): boolean {
		return this.enabled;
	}

	private log(message: string, data?: unknown): void {
		if (dev) {
			console.debug(`[telemetry] ${message}`, data ?? '');
		}
	}
}

export const telemetry = new TelemetryClient();
