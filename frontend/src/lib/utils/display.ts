/**
 * Generate a deterministic number from a string (for consistent random-like selection).
 * Useful for selecting wallpapers or other assets based on entity IDs.
 */
export function hashStringToNumber(str: string): number {
	let hash = 0;
	for (let i = 0; i < str.length; i++) {
		hash = (hash << 5) - hash + str.charCodeAt(i);
		hash = hash & hash; // Convert to 32-bit integer
	}
	return Math.abs(hash);
}

/**
 * Escape HTML special characters to prevent XSS attacks.
 * Use this when rendering user-provided or dynamic content in HTML strings.
 */
export function escapeHtml(str: string): string {
	return str
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#039;');
}

/**
 * Format a number with K/M suffixes for compact display.
 * Returns '0' if null/undefined.
 */
export function formatNumber(num: number | null | undefined): string {
	if (num === null || num === undefined) return '0';
	if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
	if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
	return num.toString();
}

/**
 * Format an ISO date string to localized short format.
 * Returns 'Unknown' if null/undefined/invalid.
 */
export function formatDate(date: string | null | undefined): string {
	if (!date) return 'Unknown';
	try {
		return new Date(date).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	} catch {
		return 'Unknown';
	}
}

/**
 * Construct Modrinth shader URL from ID.
 */
export function getModrinthUrl(modrinthId: string | null): string | null {
	return modrinthId ? `https://modrinth.com/shader/${modrinthId}` : null;
}

/**
 * Construct CurseForge shader URL from ID.
 */
export function getCurseforgeUrl(curseforgeId: string | null): string | null {
	return curseforgeId ? `https://www.curseforge.com/minecraft/shaders/${curseforgeId}` : null;
}

/**
 * Convert Minecraft biome ID to display name.
 * Strips 'minecraft:' prefix and replaces underscores with spaces.
 */
export function getBiomeDisplayName(biome: string | null): string {
	if (!biome) return 'Unknown';
	return biome.replace('minecraft:', '').replace(/_/g, ' ');
}

/**
 * Convert Minecraft dimension ID to display name.
 */
export function getDimensionDisplayName(dimension: string): string {
	return dimension.replace('minecraft:', '').replace(/_/g, ' ');
}

/**
 * Convert weather type to display name with capitalization.
 */
export function getWeatherDisplayName(weather: string): string {
	return weather.charAt(0).toUpperCase() + weather.slice(1);
}

/**
 * Format bytes to human-readable size with KiB/MiB/GiB units.
 */
export function formatBytes(bytes: number): string {
	if (bytes === 0) return '0 B';
	const k = 1024;
	const sizes = ['B', 'KiB', 'MiB', 'GiB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
