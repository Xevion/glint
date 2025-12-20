import { defineConfig, devices } from '@playwright/test';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// import dotenv from 'dotenv';
// import path from 'path';
// dotenv.config({ path: path.resolve(__dirname, '.env') });

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
	testDir: './tests',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : 4,
	reporter: 'html',
	timeout: 30000,

	use: {
		baseURL: process.env.CI ? 'http://localhost:4173' : 'http://localhost:5173',
		trace: 'retain-on-failure'
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	],

	webServer: {
		command: process.env.CI ? 'bun run build && bun run preview' : 'bun run dev',
		port: process.env.CI ? 4173 : 5173,
		reuseExistingServer: !process.env.CI,
		timeout: 120000
	}
});
