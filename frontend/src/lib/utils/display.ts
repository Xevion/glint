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
	return dimension.replace('minecraft:', '').replace('_', ' ');
}

/**
 * Convert weather type to display name with capitalization.
 */
export function getWeatherDisplayName(weather: string): string {
	return weather.charAt(0).toUpperCase() + weather.slice(1);
}
