<script lang="ts">
import { Alert } from '$lib/components/ui/alert';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import CaptureCard from '$lib/components/CaptureCard.svelte';
import {
	Counter,
	DataView,
	Grid,
	Pagination,
	Table,
	Toolbar,
	ViewToggle,
	createCursorList
} from '$lib/components/data-view';
import {
	DateRangeFilter,
	FilterCombobox,
	NumericRangeFilter,
	PopoverMultiSelect
} from '$lib/components/filters';
import { type ResultOf } from '$lib/graphql';
import type { PageData } from './$types';
import { columns } from './columns.js';
import {
	AdminCapturesQuery,
	type AdminCaptureNode,
	type CaptureFilters,
	captureFilterConfig,
	captureStatusOptions,
	captureFreshnessOptions
} from './queries';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const list = createCursorList<AdminCaptureNode, CaptureFilters>({
	key: (c) => c.id,
	initial: () => data.captures,
	query: AdminCapturesQuery,
	extract: (d: ResultOf<typeof AdminCapturesQuery>) => d.adminCaptures,
	pageSize: 50,
	filters: captureFilterConfig,
	filterVariable: 'filters',
	viewMode: 'table',
	syncUrl: true
});

const shaderOptions = $derived(data.shaders);
const sceneOptions = $derived(data.scenes);
</script>

<svelte:head><title>Captures - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Captures' }]} />

	{#if data.error}
		<Alert variant="destructive"><p>Failed to load captures: {data.error}</p></Alert>
	{/if}

	<Toolbar>
		<FilterCombobox
			placeholder="Shader"
			options={shaderOptions}
			bind:value={list.filters.shaderSlug}
		/>
		<FilterCombobox
			placeholder="Scene"
			options={sceneOptions}
			bind:value={list.filters.sceneId}
		/>
		<PopoverMultiSelect
			label="Status"
			options={captureStatusOptions}
			bind:value={list.filters.statuses}
		/>
		<PopoverMultiSelect
			label="Freshness"
			options={captureFreshnessOptions}
			bind:value={list.filters.freshness}
		/>
		<DateRangeFilter
			label="Captured"
			bind:after={list.filters.capturedAfter}
			bind:before={list.filters.capturedBefore}
		/>
		<NumericRangeFilter
			label="File Size"
			unit="MB"
			scale={1048576}
			step={0.1}
			bind:min={list.filters.minFileSize}
			bind:max={list.filters.maxFileSize}
		/>
		<div class="flex-1"></div>
		<ViewToggle modes={['table', 'tile']} bind:mode={list.viewMode} />
	</Toolbar>

	<DataView {list}>
		{#snippet empty()}
			<p class="py-12 text-center text-foreground">No captures yet.</p>
		{/snippet}

		<Table
			{columns}
			sortMapping={{ capturedAt: 'capturedAt', fileSizeBytes: 'fileSizeBytes', shaderName: 'shaderName' }}
			getRowHref={(capture: AdminCaptureNode) => `/admin/captures/${capture.id}`}
		>
			{#snippet card(capture: AdminCaptureNode)}
				<CaptureCard layout="row"
					{capture}
					title={capture.shaderName}
					subtitle={[capture.shaderVersion, capture.sceneName].filter(Boolean).join(' \u00b7 ')}
				/>
			{/snippet}
		</Table>
		<Grid href={(capture: AdminCaptureNode) => `/admin/captures/${capture.id}`} xlCols={4}>
			{#snippet card(capture: AdminCaptureNode)}
				<CaptureCard layout="tile"
					{capture}
					title={capture.shaderName}
					subtitle={[capture.shaderVersion, capture.sceneName].filter(Boolean).join(' \u00b7 ')}
				/>
			{/snippet}
		</Grid>

		<Counter noun="capture" />

		<Pagination style="load-more" />
	</DataView>
</div>
