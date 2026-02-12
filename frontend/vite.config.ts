import posthogPlugin from '@posthog/rollup-plugin';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { sveltePhosphorOptimize } from 'phosphor-svelte/vite';
import { type Plugin, defineConfig } from 'vite';
import devtoolsJson from 'vite-plugin-devtools-json';
import { jsonLogger } from './vite-plugin-json-logger';

function posthogSourceMaps(): Plugin | null {
	const apiKey = process.env.POSTHOG_PERSONAL_API_KEY;
	const projectId = process.env.POSTHOG_PROJECT_ID;
	if (!apiKey || !projectId) return null;

	return posthogPlugin({
		personalApiKey: apiKey,
		projectId,
		host: 'https://us.i.posthog.com',
		sourcemaps: {
			enabled: true,
			releaseName: 'glint-frontend',
			deleteAfterUpload: true
		}
	});
}

export default defineConfig({
	plugins: [
		jsonLogger(),
		tailwindcss(),
		sveltekit(),
		sveltePhosphorOptimize(),
		devtoolsJson(),
		posthogSourceMaps()
	],
	server: {
		watch: {
			ignored: ['**/.svelte-kit/generated/**']
		},
		proxy: {
			'/api': { target: 'http://localhost:8080', changeOrigin: true }
		}
	},

	clearScreen: false
});
