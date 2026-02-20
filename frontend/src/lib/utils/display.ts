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
 * Construct Modrinth shader URL from ID.
 */
export function getModrinthUrl(modrinthId: string | null | undefined): string | null {
	return modrinthId ? `https://modrinth.com/shader/${modrinthId}` : null;
}

/**
 * Construct CurseForge shader URL from ID.
 */
export function getCurseforgeUrl(curseforgeId: string | null | undefined): string | null {
	return curseforgeId ? `https://www.curseforge.com/minecraft/shaders/${curseforgeId}` : null;
}

/**
 * Capitalize each word in a string (Title Case).
 */
function toTitleCase(str: string): string {
	return str.replace(/\b\w/g, (c) => c.toUpperCase());
}

/**
 * Convert Minecraft biome ID to display name.
 * Strips 'minecraft:' prefix, replaces underscores with spaces, and capitalizes.
 */
export function getBiomeDisplayName(biome: string | null): string {
	if (!biome) return 'Unknown';
	return toTitleCase(biome.replace('minecraft:', '').replace(/_/g, ' '));
}

/**
 * Convert Minecraft dimension ID to display name.
 * Strips 'minecraft:' prefix, replaces underscores with spaces, and capitalizes.
 */
export function getDimensionDisplayName(dimension: string): string {
	return toTitleCase(dimension.replace('minecraft:', '').replace(/_/g, ' '));
}

/**
 * Convert weather type to display name with capitalization.
 */
export function getWeatherDisplayName(weather: string): string {
	return weather.charAt(0).toUpperCase() + weather.slice(1);
}

/**
 * Format a game versions array into a comma-separated display string.
 * Returns an em dash if null/undefined/empty.
 * Shows "first - last" range if more than 5 versions.
 */
export function formatGameVersions(versions: string[] | null | undefined): string {
	if (!versions || versions.length === 0) return '\u2014';
	if (versions.length <= 5) return versions.join(', ');
	return `${versions[0]} \u2013 ${versions[versions.length - 1]}`;
}

/**
 * Format a version string for display. Adds a "v" prefix only when the
 * version starts with a digit (e.g. "2.0.3" → "v2.0.3"). Versions that
 * already carry a prefix like "v" or "r" are returned as-is.
 */
export function formatVersion(version: string): string {
	return /^\d/.test(version) ? `v${version}` : version;
}

/**
 * Convert Minecraft time ticks to a human-readable clock string with period label.
 * Tick 0 = 6:00 AM in Minecraft. Each 1000 ticks = 1 hour.
 * Returns format like "6:00 AM (Noon)" or "7:30 PM (Night)".
 */
export function formatTimeTicks(ticks: number): string {
	const normalizedTicks = ((ticks % 24000) + 24000) % 24000;
	const hour = (Math.floor(normalizedTicks / 1000) + 6) % 24;
	const minute = Math.floor(((normalizedTicks % 1000) * 60) / 1000);
	const amPm = hour < 12 ? 'AM' : 'PM';
	const displayHour = hour === 0 ? 12 : hour > 12 ? hour - 12 : hour;
	const timeStr = `${displayHour}:${minute.toString().padStart(2, '0')} ${amPm}`;
	const label = getTimeLabel(hour);
	return `${timeStr} (${label})`;
}

function getTimeLabel(hour: number): string {
	if (hour >= 4 && hour < 6) return 'Dawn';
	if (hour >= 6 && hour < 10) return 'Morning';
	if (hour >= 10 && hour < 14) return 'Noon';
	if (hour >= 14 && hour < 18) return 'Afternoon';
	if (hour >= 18 && hour < 20) return 'Dusk';
	return 'Night';
}

const MOON_PHASES: { emoji: string; name: string }[] = [
	{ emoji: '🌕', name: 'Full Moon' },
	{ emoji: '🌖', name: 'Waning Gibbous' },
	{ emoji: '🌗', name: 'Third Quarter' },
	{ emoji: '🌘', name: 'Waning Crescent' },
	{ emoji: '🌑', name: 'New Moon' },
	{ emoji: '🌒', name: 'Waxing Crescent' },
	{ emoji: '🌓', name: 'First Quarter' },
	{ emoji: '🌔', name: 'Waxing Gibbous' }
];

/**
 * Get moon phase display info (emoji + name) for a Minecraft moon phase value (0-7).
 */
function getMoonPhaseDisplay(phase: number): { emoji: string; name: string } {
	return MOON_PHASES[phase] ?? { emoji: '🌕', name: 'Unknown' };
}

/**
 * Format a moon phase value as "emoji Name" string.
 */
export function formatMoonPhase(phase: number): string {
	const { emoji, name } = getMoonPhaseDisplay(phase);
	return `${emoji} ${name}`;
}
