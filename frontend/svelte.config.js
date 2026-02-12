import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import adapter from '@xevion/svelte-adapter-bun';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter(),
		csp: {
			mode: 'auto',
			// Report-Only: logs violations without blocking resources.
			// Move directives from reportOnly → directives once tuned.
			reportOnly: {
				'default-src': ['self'],
				'script-src': [
					'self',
					// PostHog JS SDK, toolbar, and self-hosted instance
					'https://us.posthog.com',
					'https://us-assets.i.posthog.com',
					'https://observe.xevion.dev'
				],
				// PostHog injects elements with inline event handlers (surveys, toolbar)
				'script-src-attr': ['unsafe-inline'],
				'style-src': ['self', 'unsafe-inline'],
				'img-src': [
					'self',
					'data:',
					'blob:',
					// Cloudflare R2 CDN for capture images
					'https://cdn.glint.xevion.dev',
					'https://*.r2.cloudflarestorage.com',
					// Discord avatars (OAuth user profiles)
					'https://cdn.discordapp.com',
					// Shader pack icons from platform APIs
					'https://cdn.modrinth.com',
					'https://media.forgecdn.net'
				],
				'connect-src': [
					'self',
					// PostHog event ingestion and self-hosted instance
					'https://us.posthog.com',
					'https://us-assets.i.posthog.com',
					'https://observe.xevion.dev'
				],
				'font-src': ['self'],
				'frame-ancestors': ['none'],
				'base-uri': ['self'],
				'form-action': ['self'],
				'object-src': ['none'],
				'report-uri': ['/api/csp-report']
			}
		}
	}
};

export default config;
