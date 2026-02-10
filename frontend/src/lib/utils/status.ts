/** Tailwind class strings for status badge colors using semantic tokens. */
export const statusColors: Record<string, string> = {
	running: 'bg-info/15 text-info',
	completed: 'bg-success/15 text-success',
	partial: 'bg-warning/15 text-warning',
	failed: 'bg-destructive/15 text-destructive',
	timed_out: 'bg-warning/15 text-warning',
	pending: 'bg-muted text-muted-foreground',
	skipped: 'bg-muted text-muted-foreground/70'
};

/** Fallback classes for unknown statuses. */
export const statusColorFallback = 'bg-muted text-muted-foreground';
