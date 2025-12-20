import { expect, test } from '@playwright/test';

test.describe('Smoke tests - all routes load', () => {
	test('home page loads', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('main')).toBeVisible();
		await expect(page.locator('h1')).toBeVisible();
	});

	test('scenes page loads', async ({ page }) => {
		await page.goto('/scenes');
		await expect(page.locator('main')).toBeVisible();
	});

	test('shaders page loads', async ({ page }) => {
		await page.goto('/shaders');
		await expect(page.locator('main')).toBeVisible();
	});

	test('compare page loads', async ({ page }) => {
		await page.goto('/compare');
		await expect(page.locator('main')).toBeVisible();
	});
});

test.describe('Navigation', () => {
	test('can navigate between pages', async ({ page }) => {
		await page.goto('/');

		await page.click('text=Scenes');
		await expect(page).toHaveURL('/scenes');
		await expect(page.locator('main')).toBeVisible();

		await page.click('text=Glint');
		await expect(page).toHaveURL('/');
		await expect(page.locator('main')).toBeVisible();
	});
});
