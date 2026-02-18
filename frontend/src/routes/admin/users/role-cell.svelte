<script lang="ts">
import type { Role } from '$lib/bindings';

interface Props {
	role: Role;
}

let { role }: Props = $props();

const roleColors: Record<string, string> = {
	admin: 'bg-destructive/15 text-destructive',
	agent: 'bg-info/15 text-info',
	user: 'bg-muted text-muted-foreground'
};

let colorClass = $derived(roleColors[role] || roleColors.user);
</script>

<span class="rounded px-2 py-1 text-xs font-medium {colorClass}">{role}</span>
