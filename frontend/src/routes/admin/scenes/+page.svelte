<script lang="ts">
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import {
	DataView,
	Grid,
	Search,
	Table,
	Toolbar,
	ViewToggle,
	createClientList,
	filter
} from '$lib/components/data-view';
import type { FilterDescriptor } from '$lib/components/data-view';
import { BooleanToggle } from '$lib/components/filters';
import { getDimensionDisplayName } from '$lib/utils/format';
import { MapPin } from '@lucide/svelte';
import type { PageData } from './$types';
import type { AdminScene } from './+page';
import { columns } from './columns';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions -- needs implicit index signature for Record<string, FilterDescriptor>
type SceneFilters = {
	showInactive: FilterDescriptor<boolean>;
};

const list = createClientList<AdminScene, SceneFilters>({
	items: () => data.scenes,
	search: {
		clientFilter: (scene, q) =>
			scene.name.toLowerCase().includes(q) ||
			scene.slug.toLowerCase().includes(q) ||
			(scene.description?.toLowerCase().includes(q) ?? false)
	},
	filters: {
		showInactive: filter(false)
	},
	// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- intentional boolean short-circuit: show all when filter is on, otherwise only active
	applyFilters: (scene, filters) => filters.showInactive || scene.active,
	syncUrl: true,
	viewMode: 'table'
});

const inactiveCount = $derived(data.scenes.filter((s) => !s.active).length);
</script>

<svelte:head><title>Scenes - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Scenes' }]} />

	<Toolbar>
		<Search bind:value={list.search} placeholder="Search scenes..." />
		<BooleanToggle label="Show inactive" bind:checked={list.filters.showInactive} />
		{#if list.filters.showInactive && inactiveCount > 0}
			<span class="text-sm text-foreground">
				({inactiveCount} inactive)
			</span>
		{/if}
		<div class="flex-1"></div>
		<ViewToggle modes={['table', 'grid']} bind:mode={list.viewMode} />
	</Toolbar>

	<DataView list={list} errorMessage={data.error}>
		{#snippet empty()}
			<p class="py-12 text-center text-foreground">
				{list.filters.showInactive ? 'No scenes yet.' : 'No active scenes. Enable "Show inactive" to see disabled scenes.'}
			</p>
		{/snippet}

		<Table
			{columns}
			getRowHref={(scene: AdminScene) => `/admin/scenes/${scene.id}`}
		/>
		<Grid key={(s: AdminScene) => s.id} size="small">
			{#snippet card(scene: AdminScene)}
				<a
					href="/admin/scenes/{scene.id}"
					class="group flex flex-col overflow-hidden rounded-lg border border-border bg-card transition-colors hover:border-foreground/20"
				>
					{#if scene.imagePath ?? scene.thumbhash}
						<CaptureImage
							src={scene.imagePath}
							thumbhash={scene.thumbhash}
							alt="{scene.name} preview"
							preset="card"
							class="h-full w-full object-cover"
							containerClass="aspect-video w-full"
						/>
					{:else}
						<div class="flex aspect-video w-full items-center justify-center bg-muted">
							<MapPin class="h-8 w-8 text-muted-foreground opacity-40" />
						</div>
					{/if}
					<div class="space-y-1 p-3">
						<div class="flex items-center gap-2">
							<span class="truncate text-sm font-semibold">{scene.name}</span>
							{#if !scene.active}
								<span class="shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive">Inactive</span>
							{/if}
						</div>
						{#if scene.description}
							<p class="line-clamp-2 text-xs text-muted-foreground">{scene.description}</p>
						{/if}
						<div class="flex items-center gap-1.5 border-t border-border pt-2 text-xs text-muted-foreground">
							<span>{getDimensionDisplayName(scene.dimension)}</span>
							<span>·</span>
							<span>{scene.captureCount} capture{scene.captureCount !== 1 ? 's' : ''}</span>
						</div>
					</div>
				</a>
			{/snippet}
		</Grid>
	</DataView>
</div>
