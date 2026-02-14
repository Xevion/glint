import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';
import path from 'node:path';
import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
	viteConfig,
	defineConfig({
		test: {
			projects: [
				{
					extends: true,
					test: {
						name: 'unit',
						globals: true,
						environment: 'jsdom',
						include: ['src/**/*.{test,spec}.{js,ts,svelte}'],
						exclude: ['**/node_modules/**', '**/tests/**']
					}
				},
				{
					extends: true,
					plugins: [
						storybookTest({
							configDir: path.join(import.meta.dirname, '.storybook'),
							storybookScript: 'bun run storybook --ci'
						})
					],
					resolve: {
						conditions: ['svelte', 'browser']
					},
					test: {
						name: 'storybook',
						browser: {
							enabled: true,
							provider: playwright({}),
							headless: true,
							instances: [{ browser: 'chromium' }]
						},
						setupFiles: ['./.storybook/vitest.setup.ts']
					}
				}
			]
		}
	})
);
