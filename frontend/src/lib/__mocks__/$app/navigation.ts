import type { AfterNavigate, BeforeNavigate, NavigationTarget, OnNavigate } from '@sveltejs/kit';

export async function goto(_url: string | URL): Promise<void> {
	return Promise.resolve();
}

export async function invalidate(_url: string | URL | ((url: URL) => boolean)): Promise<void> {
	return Promise.resolve();
}

export async function preloadData(_url: string | URL): Promise<void> {
	return Promise.resolve();
}

export async function preloadCode(..._urls: string[]): Promise<void> {
	return Promise.resolve();
}

export function beforeNavigate(_callback: (navigation: BeforeNavigate) => void): void {
	// Mock implementation - does nothing in tests
}

export function afterNavigate(_callback: (navigation: AfterNavigate) => void): void {
	// Mock implementation - does nothing in tests
}

export function onNavigate(_callback: (navigation: OnNavigate) => void): void {
	// Mock implementation - does nothing in tests
}

export function pushState(_url: string | URL, _state: Record<string, unknown>): Promise<void> {
	return Promise.resolve();
}

export function replaceState(_url: string | URL, _state: Record<string, unknown>): Promise<void> {
	return Promise.resolve();
}

export const navigating: NavigationTarget | null = null;
