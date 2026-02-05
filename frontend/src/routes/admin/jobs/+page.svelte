<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { invalidateAll } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { RefreshCw, Play, Pause } from '@lucide/svelte';
import AdminTable from '$lib/components/AdminTable.svelte';
import CreateJobDialog from '$lib/components/CreateJobDialog.svelte';
import JobDetailsDialog from '$lib/components/JobDetailsDialog.svelte';
import { api } from '$lib/api';
import type { JobWithDetails } from '$lib/api/endpoints/admin';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let jobs = $derived(data.jobs);
let refreshing = $state(false);
let error = $state<string | null>(null);
let autoRefresh = $state(true);
let refreshInterval: number | undefined;
let selectedJob = $state<JobWithDetails | null>(null);
let showJobDetails = $state(false);
let showDeleteConfirm = $state(false);
let jobToDelete = $state<JobWithDetails | null>(null);

const statusColors: Record<string, string> = {
	pending: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
	claimed: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
	running: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
	completed: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
	failed: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
};

const columns = [
	{ id: 'shader', key: 'shader_name', name: 'Shader' },
	{ id: 'scene_count', key: 'scene_count', name: 'Scenes' },
	{ id: 'status', key: 'status', name: 'Status' },
	{ id: 'priority', key: 'priority', name: 'Priority' },
	{ id: 'attempts', key: 'attempts', name: 'Attempts' },
	{ id: 'created_at', key: 'created_at', name: 'Created', component: 'time' as const },
	{ id: 'completed_at', key: 'completed_at', name: 'Completed', component: 'time' as const },
	{
		id: 'actions',
		key: 'id',
		name: 'Actions',
		component: 'job-actions' as const,
		onAction: handleJobAction
	}
];

async function handleJobAction(action: string, job: JobWithDetails) {
	if (action === 'view-details') {
		selectedJob = job;
		showJobDetails = true;
		return;
	}

	if (action === 'cancel') {
		const result = await api.admin.cancelJob(job.id);
		result.match({
			Ok: () => void refresh(),
			Err: (err) => (error = err.message)
		});
	} else if (action === 'retry') {
		const result = await api.admin.retryJob(job.id);
		result.match({
			Ok: () => void refresh(),
			Err: (err) => (error = err.message)
		});
	} else if (action === 'release') {
		const result = await api.admin.releaseJob(job.id);
		result.match({
			Ok: () => void refresh(),
			Err: (err) => (error = err.message)
		});
	} else if (action === 'delete') {
		jobToDelete = job;
		showDeleteConfirm = true;
	}
}

async function confirmDelete() {
	if (!jobToDelete) return;
	const result = await api.admin.deleteJob(jobToDelete.id);
	result.match({
		Ok: () => {
			jobToDelete = null;
			void refresh();
		},
		Err: (err) => {
			error = err.message;
		}
	});
}

async function refresh() {
	refreshing = true;
	error = null;
	await invalidateAll();
	refreshing = false;
}

function toggleAutoRefresh() {
	autoRefresh = !autoRefresh;
	if (autoRefresh) {
		refreshInterval = window.setInterval(() => {
			void refresh();
		}, 5000);
	} else if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = undefined;
	}
}

onMount(() => {
	if (autoRefresh) {
		refreshInterval = window.setInterval(() => {
			void refresh();
		}, 5000);
	}
});

onDestroy(() => {
	if (refreshInterval) {
		clearInterval(refreshInterval);
	}
});
</script>

<div class="space-y-4">
	<header class="flex items-center justify-between">
		<div class="flex items-baseline gap-3">
			<h1 class="text-2xl font-semibold">Jobs</h1>
			<span class="text-lg text-muted-foreground">{jobs.length}</span>
		</div>
		<div class="flex items-center gap-2">
			<Button
				variant={autoRefresh ? 'default' : 'outline'}
				size="icon"
				onclick={toggleAutoRefresh}
				title={autoRefresh ? 'Disable auto-refresh (5s)' : 'Enable auto-refresh (5s)'}
			>
				{#if autoRefresh}
					<Pause class="h-4 w-4" />
				{:else}
					<Play class="h-4 w-4" />
				{/if}
			</Button>
			<Button variant="outline" size="icon" onclick={refresh} disabled={refreshing}>
				<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
			</Button>
			<CreateJobDialog onJobCreated={refresh} />
		</div>
	</header>

	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
			Error: {error}
		</div>
	{:else if jobs.length === 0}
		<p class="text-muted-foreground">No jobs in queue.</p>
	{:else}
		<AdminTable
			data={jobs}
			{columns}
			getRowId={(j: JobWithDetails) => j.id}
		>
			{#snippet cell({ columnId, value, row }: { columnId: string; value: unknown; row: JobWithDetails })}
				{#if columnId === 'shader'}
					<div>
						<a href="/shaders/{row.shader_slug}" class="font-medium text-primary hover:underline">{row.shader_name}</a>
						<div class="text-xs text-muted-foreground">{row.shader_version}</div>
					</div>
				{:else if columnId === 'status'}
					{@const colorClass = statusColors[String(value)] || 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'}
					<span class="rounded px-2 py-1 text-xs font-medium {colorClass}">{value}</span>
				{:else if columnId === 'attempts'}
					{row.attempts}/{row.max_attempts}
				{:else}
					{value ?? '-'}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
</div>

{#if selectedJob}
	<JobDetailsDialog job={selectedJob} bind:open={showJobDetails} />
{/if}

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Job"
	description="Delete this job?"
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
