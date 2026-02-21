<script lang="ts">
import { ageMs, formatDuration } from '$lib/utils/format';
import type { AdminShader } from './queries';

interface Props {
	shader: AdminShader;
}

let { shader }: Props = $props();

function getSyncStatus(s: AdminShader): { label: string; class: string } {
	const hasLink = !!s.modrinthId || !!s.curseforgeId;
	if (!hasLink) return { label: 'No link', class: 'text-muted-foreground' };
	if (!s.lastSyncedAt) return { label: 'Never', class: 'text-warning' };

	const age = ageMs(s.lastSyncedAt);
	const days = age / (1000 * 60 * 60 * 24);
	const label = formatDuration(age);
	if (days > 7) return { label, class: 'text-destructive' };
	if (days > 1) return { label, class: 'text-warning' };
	if (days * 24 > 1) return { label, class: 'text-success' };
	return { label: 'Now', class: 'text-success' };
}

const status = $derived(getSyncStatus(shader));
</script>

<span class="text-sm font-medium {status.class}">{status.label}</span>
