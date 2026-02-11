<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { Shader } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import { AdminPageHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let shaders = $derived(data.shaders);
let refreshing = $state(false);
let error = $derived(data.error);

const columns = [
	{ id: 'name', key: 'name', name: 'Name' },
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{ id: 'description', key: 'description', name: 'Description' },
	{ id: 'sync_status', key: 'last_synced_at', name: 'Sync' },
	{ id: 'created_at', key: 'created_at', name: 'Created', component: 'time' as const }
];

function getSyncStatus(shader: Shader): { label: string; class: string } {
	const hasLink = !!shader.modrinth_id || !!shader.curseforge_id;
	if (!hasLink) return { label: 'No link', class: 'text-muted-foreground' };
	if (!shader.last_synced_at) return { label: 'Never', class: 'text-warning' };

	const days = (Date.now() - new Date(shader.last_synced_at).getTime()) / (1000 * 60 * 60 * 24);
	if (days > 7) return { label: `${Math.floor(days)}d ago`, class: 'text-destructive' };
	if (days > 1) return { label: `${Math.floor(days)}d ago`, class: 'text-warning' };
	if (days * 24 > 1)
		return { label: `${Math.floor(days * 24)}h ago`, class: 'text-green-600 dark:text-green-400' };
	return { label: 'Just now', class: 'text-green-600 dark:text-green-400' };
}

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}
</script>

<svelte:head><title>Shaders - Glint</title></svelte:head>

<div class="space-y-4">
	<AdminPageHeader title="Shaders" count={shaders.length} {refreshing} onrefresh={refresh}>
		{#snippet actions()}
			<AdoptShaderDialog onShaderAdopted={() => refresh()} />
		{/snippet}
	</AdminPageHeader>

	{#if error}
		<Alert variant="destructive">Error: {error}</Alert>
	{:else if shaders.length === 0}
		<p class="text-foreground">No shaders yet.</p>
	{:else}
		<AdminTable
			data={shaders}
			{columns}
			onRowClick={(shader: Shader) => goto(`/admin/shaders/${shader.id}`)}
			getRowId={(s: Shader) => s.id}
		>
			{#snippet cell({ columnId, value, row })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
				{:else if columnId === 'description'}
					{#if value}
						<span class="line-clamp-1">{value}</span>
					{:else}
						<span class="text-muted-foreground">-</span>
					{/if}
				{:else if columnId === 'sync_status'}
					{@const status = getSyncStatus(row as Shader)}
					<span class="text-xs font-medium {status.class}">{status.label}</span>
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
