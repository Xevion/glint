import devtoolsJson from 'vite-plugin-devtools-json';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { sveltePhosphorOptimize } from 'phosphor-svelte/vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), sveltePhosphorOptimize(), devtoolsJson()],

	server: {
		proxy: {
			'/api': { target: 'http://localhost:8080', changeOrigin: true },
			'/health': { target: 'http://localhost:8080', changeOrigin: true }
		}
	},

	clearScreen: false
});
