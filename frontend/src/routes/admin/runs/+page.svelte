<script lang="ts">
import { Alert } from '$lib/components/ui/alert';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import {
	DataView,
	Grid,
	Pagination,
	Table,
	Toolbar,
	ViewToggle,
	createCursorList
} from '$lib/components/data-view';
import { DateRangeFilter, NumericRangeFilter, PopoverMultiSelect } from '$lib/components/filters';
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import { formatDuration } from '$lib/utils/format';
import { type ResultOf } from '$lib/graphql';
import type { PageData } from './$types';
import { columns } from './columns.js';
import { AdminRunsQuery, type RunNode, type RunFilters, runFilterConfig } from './queries.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

const statusOptions = [
	{ value: 'RUNNING', label: 'Running' },
	{ value: 'COMPLETED', label: 'Completed' },
	{ value: 'PARTIAL', label: 'Partial' },
	{ value: 'FAILED', label: 'Failed' },
	{ value: 'TIMED_OUT', label: 'Timed Out' }
];

const list = createCursorList<RunNode, RunFilters>({
	initial: () => data.runs,
	query: AdminRunsQuery,
	extract: (d: ResultOf<typeof AdminRunsQuery>) => d.adminCaptureRuns,
	pageSize: 25,
	filters: runFilterConfig,
	filterVariable: 'filters',
	viewMode: 'table',
	syncUrl: true
});
</script>

{#snippet runCard(run: RunNode)}
	{@const remaining = run.totalItems - run.completedItems - run.failedItems - run.skippedItems}
	<div class="flex items-start justify-between gap-3">
		<div class="min-w-0 flex-1 space-y-1">
			<div class="flex items-center gap-2">
				<StatusBadge status={run.status.toLowerCase() as StatusBadgeStatus}>
					{run.status.toLowerCase().replace(/_/g, ' ')}
				</StatusBadge>
				<span class="text-xs text-muted-foreground">
					{run.agentId ?? '—'}
				</span>
			</div>
			{#if run.totalItems > 0}
				<div class="flex h-2 w-full overflow-hidden rounded-full bg-muted">
					{#if run.completedItems > 0}
						<div
							class="bg-success"
							style="width: {(run.completedItems / run.totalItems) * 100}%"
						></div>
					{/if}
					{#if run.failedItems > 0}
						<div
							class="bg-destructive"
							style="width: {(run.failedItems / run.totalItems) * 100}%"
						></div>
					{/if}
					{#if run.skippedItems > 0}
						<div
							class="bg-muted-foreground/30"
							style="width: {(run.skippedItems / run.totalItems) * 100}%"
						></div>
					{/if}
					{#if remaining > 0}
						<div
							class={run.status === 'TIMED_OUT' ? 'bg-warning' : 'bg-info'}
							style="width: {(remaining / run.totalItems) * 100}%"
						></div>
					{/if}
				</div>
				<p class="text-xs text-muted-foreground">
					{run.completedItems}/{run.totalItems} done
					{#if run.failedItems > 0}, {run.failedItems} failed{/if}
				</p>
			{/if}
		</div>
		<div class="shrink-0 text-right text-xs text-muted-foreground">
			{#if run.durationSecs != null}
				{formatDuration(run.durationSecs * 1000)}
			{:else}
				In progress
			{/if}
		</div>
	</div>
{/snippet}

<svelte:head><title>Runs - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Capture Runs' }]} />

	{#if data.error}
		<Alert variant="destructive"><p>Failed to load runs: {data.error}</p></Alert>
	{/if}

	<Toolbar>
		<PopoverMultiSelect
			label="Status"
			options={statusOptions}
			bind:value={list.filters.statuses}
		/>
		<DateRangeFilter
			label="Started"
			bind:after={list.filters.startedAfter}
			bind:before={list.filters.startedBefore}
		/>
		<NumericRangeFilter
			label="Duration"
			unit="s"
			step={1}
			bind:min={list.filters.minDurationSecs}
			bind:max={list.filters.maxDurationSecs}
		/>
		<div class="flex-1"></div>
		<ViewToggle modes={['table', 'tile']} bind:mode={list.viewMode} />
	</Toolbar>

	<DataView {list}>
		{#snippet empty()}
			<p class="py-12 text-center text-foreground">No capture runs yet.</p>
		{/snippet}

		<Table
			{columns}
			sortMapping={{ startedAt: 'startedAt', duration: 'durationSecs', status: 'status', totalItems: 'totalItems' }}
			getRowHref={(run: RunNode) => `/admin/runs/${String(run.id)}`}
		>
			{#snippet card(run: RunNode)}
				{@render runCard(run)}
			{/snippet}
		</Table>
		<Grid href={(run: RunNode) => `/admin/runs/${String(run.id)}`} key={(run: RunNode) => String(run.id)}>
			{#snippet card(run: RunNode)}
				<div class="p-4">
					{@render runCard(run)}
				</div>
			{/snippet}
		</Grid>

		<Pagination style="load-more" />
	</DataView>
</div>
