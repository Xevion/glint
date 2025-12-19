import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import tailwindcss from '@tailwindcss/vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import path from 'path';

export default defineConfig({
	plugins: [
		tailwindcss(),
		svelte({ hot: false }),
		paraglideVitePlugin({ project: './project.inlang', outdir: './src/lib/paraglide' })
	],

	resolve: {
		alias: {
			$lib: path.resolve('./src/lib'),
			'$app/paths': path.resolve('./src/lib/__mocks__/$app/paths.ts'),
			'$app/environment': path.resolve('./src/lib/__mocks__/$app/environment.ts'),
			'$app/navigation': path.resolve('./src/lib/__mocks__/$app/navigation.ts'),
			'$app/state': path.resolve('./src/lib/__mocks__/$app/state.ts')
		}
	},

	test: {
		expect: { requireAssertions: true },

		projects: [
			{
				test: {
					name: 'client',

					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},

					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**']
				}
			},

			{
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
