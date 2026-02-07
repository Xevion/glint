<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { Shader } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import { Button } from '$lib/components/ui/button';
import { RefreshCw } from '@lucide/svelte';
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
	await invalidateAll();
	refreshing = false;
}
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Shaders</h1>
			<span class="text-lg text-muted-foreground">{shaders.length}</span>
		</div>
		<div class="flex items-center gap-2">
			<AdoptShaderDialog onShaderAdopted={refresh} />
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		</div>
	</header>

	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if shaders.length === 0}
		<p class="text-muted-foreground">No shaders yet.</p>
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
