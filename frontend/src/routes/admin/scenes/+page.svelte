<script lang="ts">
import { goto, invalidateAll } from '$app/navigation';
import type { SceneWithWorld } from '$lib/bindings';
import AdminTable from '$lib/components/AdminTable.svelte';
import { Button } from '$lib/components/ui/button';
import * as Checkbox from '$lib/components/ui/checkbox';
import { RefreshCw } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let scenes: SceneWithWorld[] = $derived(data.scenes);
let refreshing = $state(false);
let showInactive = $state(false);
let error = $derived(data.error);

const filteredScenes = $derived(showInactive ? scenes : scenes.filter((s) => s.active));

const columns = [
	{ id: 'name', key: 'name', name: 'Name' },
	{ id: 'slug', key: 'slug', name: 'Slug' },
	{ id: 'world', key: 'world_name', name: 'World' },
	{ id: 'dimension', key: 'dimension', name: 'Dimension' },
	{ id: 'weather', key: 'weather', name: 'Weather' },
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
			<h1 class="text-2xl font-semibold">Scenes</h1>
			<span class="text-lg text-muted-foreground">{filteredScenes.length}</span>
			{#if showInactive && scenes.some((s) => !s.active)}
				<span class="text-sm text-muted-foreground">
					({scenes.filter((s) => !s.active).length} inactive)
				</span>
			{/if}
		</div>
		<div class="flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm">
				<Checkbox.Root
					checked={showInactive}
					onCheckedChange={(v) => (showInactive = v === true)}
				/>
				<span>Show inactive</span>
			</label>
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
		</div>
	</header>

	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if filteredScenes.length === 0}
		<p class="text-muted-foreground">
			{showInactive
				? 'No scenes yet.'
				: 'No active scenes. Enable "Show inactive" to see disabled scenes.'}
		</p>
	{:else}
		<AdminTable
			data={filteredScenes}
			{columns}
			onRowClick={(scene: SceneWithWorld) => goto(`/admin/scenes/${scene.id}`)}
			getRowId={(s: SceneWithWorld) => s.id}
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: SceneWithWorld })}
				{#if columnId === 'name'}
					<span class="font-medium">{value}</span>
					{#if !row.active}
						<span
							class="ml-2 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
							>Inactive</span
						>
					{/if}
				{:else if columnId === 'world'}
					{#if value}
						<span>{value}</span>
					{:else}
						<span class="text-muted-foreground">-</span>
					{/if}
				{:else if columnId === 'dimension'}
					<span class="text-xs"
						>{typeof value === 'string' ? value.split(':').pop() : value}</span
					>
				{:else if columnId === 'weather'}
					<span class="text-xs capitalize">{value}</span>
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>
