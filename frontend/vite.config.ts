import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { sveltePhosphorOptimize } from 'phosphor-svelte/vite';
import { defineConfig } from 'vite';
import devtoolsJson from 'vite-plugin-devtools-json';
import { jsonLogger } from './vite-plugin-json-logger';

export default defineConfig({
	plugins: [jsonLogger(), tailwindcss(), sveltekit(), sveltePhosphorOptimize(), devtoolsJson()],
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
