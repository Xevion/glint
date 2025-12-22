/**
 * Ensures a timestamp string is treated as UTC by appending 'Z' if needed
 * SQLite datetime('now') returns "YYYY-MM-DD HH:MM:SS" without timezone,
 * which browsers interpret as local time. This fixes that.
 */
export function ensureUtc(timestamp: string): string {
	if (!timestamp) return timestamp;

	// Already has timezone info (Z or +/-offset)
	if (timestamp.endsWith('Z') || /[+-]\d{2}:\d{2}$/.test(timestamp)) {
		return timestamp;
	}

	// SQLite format: "2024-12-21 16:47:59" -> "2024-12-21T16:47:59Z"
	// Replace space with T if needed, then append Z
	return timestamp.replace(' ', 'T') + 'Z';
}
