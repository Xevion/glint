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
 * Format a Date with time for tooltip display (e.g., "Jan 15, 2:00 PM").
 */
export function formatDatetime(date: Date): string {
	return date.toLocaleDateString(undefined, {
		month: 'short',
		day: 'numeric',
		hour: 'numeric',
		minute: '2-digit'
	});
}

/**
 * Format the duration of a capture run as a human-readable string.
 * Returns "In progress" if no completion time is set.
 */
export function formatDuration(run: {
	started_at: string;
	completed_at?: string | null;
}): string {
	if (!run.completed_at) return 'In progress';
	const start = new Date(run.started_at).getTime();
	const end = new Date(run.completed_at).getTime();
	const seconds = Math.round((end - start) / 1000);
	if (seconds < 60) return `${seconds}s`;
	const minutes = Math.floor(seconds / 60);
	const remaining = seconds % 60;
	return `${minutes}m ${remaining}s`;
}
