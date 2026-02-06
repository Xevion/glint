import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { sveltePhosphorOptimize } from 'phosphor-svelte/vite';
import { defineConfig } from 'vite';
import devtoolsJson from 'vite-plugin-devtools-json';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), sveltePhosphorOptimize(), devtoolsJson()],
	server: {
		watch: {
			ignored: ['**/.svelte-kit/generated/**']
		},
		proxy: {
			'/api': { target: 'http://localhost:8080', changeOrigin: true },
			'/health': { target: 'http://localhost:8080', changeOrigin: true }
		}
	},

	clearScreen: false
});
