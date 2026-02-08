/**
 * Format a byte count into a human-readable string (e.g., "1.23 GB").
 */
export function formatBytes(bytes: number, decimals = 2): string {
	if (bytes === 0) return '0 B';
	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
}

/**
 * Create a stateful axis tick formatter that deduplicates date labels.
 * Shows "Jan 15" for the first tick of each day, then "4 PM", "8 PM" for
 * subsequent same-day ticks. Resets when a new day is encountered.
 */
export function createAxisDateFormatter(): (date: Date) => string {
	let lastDay = '';
	return (date: Date) => {
		const day = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
		if (day !== lastDay) {
			lastDay = day;
			return day;
		}
		return date.toLocaleTimeString('en-US', { hour: 'numeric' });
	};
}

/**
 * Format a Date with time for tooltip display (e.g., "Jan 15, 2:00 PM").
 */
export function formatDatetime(date: Date): string {
	return date.toLocaleDateString('en-US', {
		month: 'short',
		day: 'numeric',
		hour: 'numeric',
		minute: '2-digit'
	});
}
