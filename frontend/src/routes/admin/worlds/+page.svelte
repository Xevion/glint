<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { World } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import WorldUploadDialog from '$lib/components/WorldUploadDialog.svelte';
import { Button } from '$lib/components/ui/button';
import { formatBytes } from '$lib/utils/display';
import { RefreshCw } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let worlds = $derived(data.worlds);
let refreshing = $state(false);
let error = $derived(data.error);

const columns = [
	{ id: 'name', key: 'name', name: 'Name' },
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{ id: 'minecraft_version', key: 'minecraft_version', name: 'MC Version' },
	{ id: 'size', key: 'size_bytes', name: 'Size' },
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
			<h1 class="text-2xl font-semibold">Worlds</h1>
			<span class="text-lg text-muted-foreground">{worlds.length}</span>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
			<WorldUploadDialog onWorldCreated={refresh} />
		</div>
	</header>

	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if worlds.length === 0}
		<p class="text-muted-foreground">No worlds uploaded yet.</p>
	{:else}
		<AdminTable
			data={worlds}
			{columns}
			onRowClick={(world: World) => goto(`/admin/worlds/${world.id}`)}
			getRowId={(w: World) => w.id}
		>
			{#snippet cell({ columnId, value })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
				{:else if columnId === 'minecraft_version'}
					<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{value}</code>
				{:else if columnId === 'size'}
					{value ? formatBytes(value as number) : '-'}
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
