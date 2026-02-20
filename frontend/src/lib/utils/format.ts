/**
 * Normalize various date inputs to a Date object.
 * Accepts ISO strings or existing Date instances.
 */
export function toDate(input: string | Date): Date {
	return typeof input === 'string' ? new Date(input) : input;
}

/**
 * Human-readable date: "Jan 15, 2024".
 * Uses the browser's locale for formatting.
 */
export function formatDate(input: string | Date): string {
	return toDate(input).toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric'
	});
}

/**
 * Human-readable datetime: "Jan 15, 2:00 PM".
 * Uses the browser's locale for formatting.
 */
export function formatDatetime(input: string | Date): string {
	return toDate(input).toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		hour: 'numeric',
		minute: '2-digit'
	});
}

/**
 * Full datetime with time-only display: "2:00:00 PM".
 * Useful for error timestamps and log entries.
 */
export function formatTime(input: string | Date): string {
	return toDate(input).toLocaleTimeString();
}

/**
 * Compact relative time: "just now", "5m ago", "3d ago".
 * Pure function for non-live contexts (tooltips, status text).
 * For live-updating DOM elements, use the TimeAgo component instead.
 */
export function formatRelativeTime(input: string | Date): string {
	const ms = Date.now() - toDate(input).getTime();
	const seconds = Math.floor(ms / 1000);
	if (seconds < 60) return 'just now';
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	const days = Math.floor(hours / 24);
	if (days < 30) return `${days}d ago`;
	const months = Math.floor(days / 30);
	return `${months}mo ago`;
}

/**
 * Returns the age of a timestamp in milliseconds.
 * Useful when you need the raw age for threshold logic alongside formatted output.
 */
export function ageMs(input: string | Date): number {
	return Date.now() - toDate(input).getTime();
}

/**
 * Adaptive duration formatting from milliseconds.
 * Automatically scales units based on magnitude:
 *   0ms      → "0ms"
 *   500ms    → "500ms"
 *   1200ms   → "1.2s"
 *   72000ms  → "1m 12s"
 *   3723000  → "1h 2m"
 *   90061000 → "1d 1h"
 */
export function formatDuration(ms: number): string {
	if (ms < 0) ms = 0;
	if (ms < 1000) return `${Math.round(ms)}ms`;
	const totalSeconds = ms / 1000;
	if (totalSeconds < 60) return `${totalSeconds.toFixed(1)}s`;
	const totalMinutes = Math.floor(totalSeconds / 60);
	const remainingSeconds = Math.round(totalSeconds % 60);
	if (totalMinutes < 60) return `${totalMinutes}m ${remainingSeconds}s`;
	const totalHours = Math.floor(totalMinutes / 60);
	const remainingMinutes = totalMinutes % 60;
	if (totalHours < 24) return `${totalHours}h ${remainingMinutes}m`;
	const days = Math.floor(totalHours / 24);
	const remainingHours = totalHours % 24;
	return `${days}d ${remainingHours}h`;
}

/**
 * Format elapsed duration from a start/end ISO timestamp pair.
 * Returns "In progress" if no completion time is set.
 */
export function formatElapsedDuration(run: {
	started_at: string;
	completed_at?: string | null;
}): string {
	if (!run.completed_at) return 'In progress';
	const ms = new Date(run.completed_at).getTime() - new Date(run.started_at).getTime();
	return formatDuration(ms);
}

const EM_DASH = '\u2014';

/**
 * Compact number with K/M suffixes: 1234 → "1.2K", 1.5M → "1.5M".
 * Returns '0' for null/undefined.
 */
export function formatNumber(num: number | null | undefined): string {
	if (num == null) return '0';
	if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
	if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
	return num.toString();
}

/**
 * Locale-aware integer formatting with grouping separators: 1234 → "1,234".
 * Returns an em dash for null/undefined.
 */
export function formatCount(num: number | null | undefined): string {
	if (num == null) return EM_DASH;
	return num.toLocaleString();
}

/**
 * Fixed decimal places with null handling.
 * Returns an em dash for null/undefined values.
 */
export function formatDecimal(value: number | null | undefined, places = 2): string {
	if (value == null) return EM_DASH;
	return value.toFixed(places);
}

/**
 * Format a ratio (0-1) as a percentage: 0.567 → "56.7%".
 * Returns an em dash for null/undefined values.
 */
export function formatPercent(ratio: number | null | undefined, places = 1): string {
	if (ratio == null) return EM_DASH;
	return `${(ratio * 100).toFixed(places)}%`;
}

/**
 * Format a number with a unit suffix: formatMetric(16.67, 'ms') → "16.67ms".
 * Returns an em dash for null/undefined values.
 */
export function formatMetric(value: number | null | undefined, unit: string, places = 2): string {
	if (value == null) return EM_DASH;
	return `${value.toFixed(places)}${unit}`;
}

/**
 * Format a byte count into a human-readable string using binary units (e.g., "1.23 GiB").
 */
export function formatBytes(bytes: number, decimals = 2): string {
	if (bytes <= 0) return '0 B';
	const k = 1024;
	const sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
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
	if (!versions || versions.length === 0) return EM_DASH;
	if (versions.length <= 5) return versions.join(', ');
	return `${versions[0]} \u2013 ${versions[versions.length - 1]}`;
}

/**
 * Format a version string for display. Adds a "v" prefix only when the
 * version starts with a digit (e.g. "2.0.3" → "v2.0.3").
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
	{ emoji: '\u{1F315}', name: 'Full Moon' },
	{ emoji: '\u{1F316}', name: 'Waning Gibbous' },
	{ emoji: '\u{1F317}', name: 'Third Quarter' },
	{ emoji: '\u{1F318}', name: 'Waning Crescent' },
	{ emoji: '\u{1F311}', name: 'New Moon' },
	{ emoji: '\u{1F312}', name: 'Waxing Crescent' },
	{ emoji: '\u{1F313}', name: 'First Quarter' },
	{ emoji: '\u{1F314}', name: 'Waxing Gibbous' }
];

function getMoonPhaseDisplay(phase: number): { emoji: string; name: string } {
	return MOON_PHASES[phase] ?? { emoji: '\u{1F315}', name: 'Unknown' };
}

/**
 * Format a moon phase value as "emoji Name" string.
 */
export function formatMoonPhase(phase: number): string {
	const { emoji, name } = getMoonPhaseDisplay(phase);
	return `${emoji} ${name}`;
}
