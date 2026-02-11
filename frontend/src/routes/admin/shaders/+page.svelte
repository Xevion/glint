<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { Shader } from '$lib/bindings';
import { AdminPageHeader } from '$lib/components/admin';
import AdminTable from '$lib/components/AdminTable.svelte';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
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
	{ id: 'created_at', key: 'created_at', name: 'Created', component: 'time' as const }
];

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
			{#snippet cell({ columnId, value })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
				{:else if columnId === 'description'}
					{#if value}
						<span class="line-clamp-1">{value}</span>
					{:else}
						<span class="text-muted-foreground">-</span>
					{/if}
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
