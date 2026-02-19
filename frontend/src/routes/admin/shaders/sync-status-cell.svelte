<script lang="ts">
import type { AdminShader } from './+page';

interface Props {
	shader: AdminShader;
}

let { shader }: Props = $props();

function getSyncStatus(s: AdminShader): { label: string; class: string } {
	const hasLink = !!s.modrinthId || !!s.curseforgeId;
	if (!hasLink) return { label: 'No link', class: 'text-muted-foreground' };
	if (!s.lastSyncedAt) return { label: 'Never', class: 'text-warning' };

	const days = (Date.now() - new Date(s.lastSyncedAt).getTime()) / (1000 * 60 * 60 * 24);
	if (days > 7) return { label: `${Math.floor(days)}d`, class: 'text-destructive' };
	if (days > 1) return { label: `${Math.floor(days)}d`, class: 'text-warning' };
	if (days * 24 > 1) return { label: `${Math.floor(days * 24)}h`, class: 'text-success' };
	return { label: 'Now', class: 'text-success' };
}

const status = $derived(getSyncStatus(shader));
</script>

<span class="text-sm font-medium {status.class}">{status.label}</span>
