<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { RefreshCw, Play, Pause } from '@lucide/svelte';
	import TimeAgo from '$lib/components/time-ago.svelte';
	import AdminTable from '$lib/components/admin-table.svelte';
	import CreateJobDialog from '$lib/components/create-job-dialog.svelte';
	import JobDetailsDialog from '$lib/components/job-details-dialog.svelte';
	import { api } from '$lib/api';
	import type { Shader, Scene, CaptureWithContext } from '$lib/api';
	import type { JobWithDetails } from '$lib/api/endpoints/admin';
	import { escapeHtml } from '$lib/utils/display';

	let shaders = $state<Shader[]>([]);
	let scenes = $state<Scene[]>([]);
	let captures = $state<CaptureWithContext[]>([]);
	let jobs = $state<JobWithDetails[]>([]);
	let loading = $state(true);
	let errors = $state<Record<string, string>>({});
	let autoRefresh = $state(true);
	let refreshing = $state(false);
	let refreshTimeout: number | undefined;
	let isLoadingData = $state(false);
	let healthStatus = $state<'ok' | 'error' | 'checking'>('checking');
	let lastRefreshed = $state<Date | null>(null);
	let selectedJob = $state<JobWithDetails | null>(null);
	let showJobDetails = $state(false);

	let refreshInterval: number | undefined;

	// Shader table columns
	const shaderColumns = [
		{ id: 'name', key: 'name', name: 'Name' },
		{ id: 'slug', key: 'slug', name: 'Slug' },
		{
			id: 'description',
			key: 'description',
			name: 'Description',
			render: (value: string | null) => value ?? 'No description'
		},
		{
			id: 'created_at',
			key: 'created_at',
			name: 'Created',
			component: 'time' as const
		},
		{
			id: 'actions',
			key: 'slug',
			name: 'Actions',
			component: 'link-button' as const,
			href: (row: Shader) => `/shaders/${row.slug}`
		}
	];

	// Scene table columns
	const sceneColumns = [
		{ id: 'name', key: 'name', name: 'Name' },
		{ id: 'slug', key: 'slug', name: 'Slug' },
		{
			id: 'description',
			key: 'description',
			name: 'Description',
			render: (value: string | null) => value ?? 'No description'
		},
		{
			id: 'created_at',
			key: 'created_at',
			name: 'Created',
			component: 'time' as const
		},
		{
			id: 'actions',
			key: 'slug',
			name: 'Actions',
			component: 'link-button' as const,
			href: (row: Scene) => `/scenes/${row.slug}`
		}
	];

	// Capture table columns
	const captureColumns = [
		{
			id: 'preview',
			key: 'screenshot_url',
			name: 'Preview',
			render: (value: string | null, row: CaptureWithContext) => {
				if (value) {
					const safeUrl = escapeHtml(value);
					return `<img src="${safeUrl}" alt="Capture preview" class="h-12 w-20 rounded object-cover" />`;
				} else if (row.screenshot_path) {
					return '<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">No URL</div>';
				}
				return '<div class="flex h-12 w-20 items-center justify-center rounded bg-muted text-xs text-muted-foreground">N/A</div>';
			}
		},
		{
			id: 'shader',
			key: 'shader_name',
			name: 'Shader',
			render: (_value: unknown, row: CaptureWithContext) => {
				const safeName = escapeHtml(row.shader_name);
				const safeSlug = escapeHtml(row.shader_slug);
				const safeVersion = escapeHtml(row.shader_version);
				return `<div><a href="/shaders/${safeSlug}" class="font-medium text-primary hover:underline">${safeName}</a><div class="text-xs text-muted-foreground">${safeVersion}</div></div>`;
			}
		},
		{ id: 'scene_id', key: 'scene_id', name: 'Scene' },
		{
			id: 'resolution',
			key: 'resolution_width',
			name: 'Resolution',
			render: (_value: unknown, row: CaptureWithContext) =>
				row.resolution_width && row.resolution_height
					? `${row.resolution_width}x${row.resolution_height}`
					: '-'
		},
		{
			id: 'captured_at',
			key: 'captured_at',
			name: 'Captured',
			component: 'time' as const
		},
		{
			id: 'actions',
			key: 'id',
			name: 'Actions',
			component: 'delete-button' as const
		}
	];

	// Job table columns
	const jobColumns = [
		{
			id: 'shader',
			key: 'shader_name',
			name: 'Shader',
			render: (_value: unknown, row: JobWithDetails) => {
				const safeName = escapeHtml(row.shader_name);
				const safeSlug = escapeHtml(row.shader_slug);
				const safeVersion = escapeHtml(row.shader_version);
				return `<div><a href="/shaders/${safeSlug}" class="font-medium text-primary hover:underline">${safeName}</a><div class="text-xs text-muted-foreground">${safeVersion}</div></div>`;
			}
		},
		{ id: 'scene_count', key: 'scene_count', name: 'Scenes' },
		{
			id: 'status',
			key: 'status',
			name: 'Status',
			render: (value: string) => {
				const colorMap: Record<string, string> = {
					pending: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
					claimed: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
					running: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
					completed: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
					failed: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
				};
				const colorClass =
					colorMap[value] || 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300';
				return `<span class="rounded px-2 py-1 text-xs font-medium ${colorClass}">${value}</span>`;
			}
		},
		{ id: 'priority', key: 'priority', name: 'Priority' },
		{
			id: 'attempts',
			key: 'attempts',
			name: 'Attempts',
			render: (_value: unknown, row: JobWithDetails) => `${row.attempts}/${row.max_attempts}`
		},
		{
			id: 'created_at',
			key: 'created_at',
			name: 'Created',
			component: 'time' as const
		},
		{
			id: 'completed_at',
			key: 'completed_at',
			name: 'Completed',
			component: 'time' as const
		},
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
			if (result.isOk) {
				void loadJobs();
			} else {
				errors.jobs = result.error.message;
			}
		} else if (action === 'retry') {
			const result = await api.admin.retryJob(job.id);
			if (result.isOk) {
				void loadJobs();
			} else {
				errors.jobs = result.error.message;
			}
		} else if (action === 'release') {
			const result = await api.admin.releaseJob(job.id);
			if (result.isOk) {
				void loadJobs();
			} else {
				errors.jobs = result.error.message;
			}
		} else if (action === 'delete') {
			const result = await api.admin.deleteJob(job.id);
			if (result.isOk) {
				void loadJobs();
			} else {
				errors.jobs = result.error.message;
			}
		}
	}

	async function loadShaders() {
		const result = await api.shaders.list();
		if (result.isOk) {
			shaders = result.value;
		} else {
			errors.shaders = result.error.message;
		}
	}

	async function loadScenes() {
		const result = await api.scenes.list();
		if (result.isOk) {
			scenes = result.value;
		} else {
			errors.scenes = result.error.message;
		}
	}

	async function loadCaptures() {
		const result = await api.captures.list();
		if (result.isOk) {
			captures = result.value;
		} else {
			errors.captures = result.error.message;
		}
	}

	async function loadJobs() {
		const result = await api.admin.listJobs();
		if (result.isOk) {
			jobs = result.value;
		} else {
			errors.jobs = result.error.message;
		}
	}

	async function loadHealth() {
		const result = await api.admin.health();
		if (result.isOk) {
			healthStatus = 'ok';
		} else {
			healthStatus = 'error';
			errors.health = result.error.message;
		}
	}

	async function loadData() {
		// Prevent overlapping refresh calls
		if (isLoadingData) return;
		isLoadingData = true;

		// Clear any pending timeout
		if (refreshTimeout) {
			clearTimeout(refreshTimeout);
			refreshTimeout = undefined;
		}

		refreshing = true;
		errors = {};

		try {
			await Promise.all([loadShaders(), loadScenes(), loadCaptures(), loadJobs(), loadHealth()]);
			lastRefreshed = new Date();
		} finally {
			loading = false;
			isLoadingData = false;
			// Keep spinner visible briefly for visual feedback
			refreshTimeout = window.setTimeout(() => {
				refreshing = false;
			}, 500);
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			refreshInterval = window.setInterval(() => {
				void loadData();
			}, 5000);
		} else if (refreshInterval) {
			clearInterval(refreshInterval);
			refreshInterval = undefined;
		}
	}

	onMount(() => {
		void loadData();
		// Start auto-refresh since it's enabled by default
		if (autoRefresh) {
			refreshInterval = window.setInterval(() => {
				void loadData();
			}, 5000);
		}
	});

	onDestroy(() => {
		if (refreshInterval) {
			clearInterval(refreshInterval);
		}
		if (refreshTimeout) {
			clearTimeout(refreshTimeout);
		}
	});
</script>

<div class="container mx-auto px-4 py-8">
	<div class="mb-8 flex items-center justify-between">
		<div class="flex items-center gap-4">
			<h1 class="text-3xl font-bold">Admin Panel</h1>
			<div
				class="flex h-3 w-3 items-center justify-center rounded-full"
				class:bg-green-500={healthStatus === 'ok'}
				class:bg-red-500={healthStatus === 'error'}
				class:bg-gray-400={healthStatus === 'checking'}
				title={healthStatus === 'ok'
					? 'API healthy'
					: healthStatus === 'error'
						? `API error: ${errors.health || 'Unknown'}`
						: 'Checking API health'}
			></div>
		</div>
		<div class="flex items-center gap-4">
			{#if lastRefreshed}
				<div class="text-sm text-muted-foreground">
					{#if refreshing}
						Refreshing...
					{:else}
						Last updated <TimeAgo timestamp={lastRefreshed} />
					{/if}
				</div>
			{/if}
			<div class="flex gap-2">
				<Button
					variant="outline"
					size="icon"
					onclick={toggleAutoRefresh}
					class={autoRefresh ? 'bg-primary text-primary-foreground' : ''}
					title={autoRefresh ? 'Disable auto-refresh (5s)' : 'Enable auto-refresh (5s)'}
				>
					{#if autoRefresh}
						<Pause class="h-4 w-4" />
					{:else}
						<Play class="h-4 w-4" />
					{/if}
				</Button>
				<Button
					variant="outline"
					size="icon"
					onclick={loadData}
					disabled={refreshing}
					title="Refresh data"
				>
					<RefreshCw class={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} />
				</Button>
			</div>
		</div>
	</div>

	{#if loading}
		<div class="text-center text-muted-foreground">Loading...</div>
	{:else}
		<div class="grid gap-6">
			<!-- Shaders Section -->
			<section class="rounded-lg border bg-card p-6">
				<div class="mb-4 flex items-baseline gap-3">
					<h2 class="text-2xl font-semibold">Shaders</h2>
					<span class="text-lg text-muted-foreground">{shaders.length}</span>
				</div>
				{#if errors.shaders}
					<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
						Error: {errors.shaders}
					</div>
				{:else}
					<AdminTable columns={shaderColumns} data={shaders} />
				{/if}
			</section>

			<!-- Scenes Section -->
			<section class="rounded-lg border bg-card p-6">
				<div class="mb-4 flex items-baseline gap-3">
					<h2 class="text-2xl font-semibold">Scenes</h2>
					<span class="text-lg text-muted-foreground">{scenes.length}</span>
				</div>
				{#if errors.scenes}
					<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
						Error: {errors.scenes}
					</div>
				{:else}
					<AdminTable columns={sceneColumns} data={scenes} />
				{/if}
			</section>

			<!-- Captures Section -->
			<section class="rounded-lg border bg-card p-6">
				<div class="mb-4 flex items-baseline gap-3">
					<h2 class="text-2xl font-semibold">Captures</h2>
					<span class="text-lg text-muted-foreground">{captures.length}</span>
				</div>
				{#if errors.captures}
					<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
						Error: {errors.captures}
					</div>
				{:else if captures.length === 0}
					<p class="text-muted-foreground">No captures yet.</p>
				{:else}
					<AdminTable columns={captureColumns} data={captures} />
				{/if}
			</section>

			<!-- Jobs Section -->
			<section class="rounded-lg border bg-card p-6">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-baseline gap-3">
						<h2 class="text-2xl font-semibold">Jobs</h2>
						<span class="text-lg text-muted-foreground">{jobs.length}</span>
					</div>
					<CreateJobDialog onJobCreated={loadData} />
				</div>
				{#if errors.jobs}
					<div class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
						Error: {errors.jobs}
					</div>
				{:else if jobs.length === 0}
					<p class="text-muted-foreground">No jobs in queue.</p>
				{:else}
					<AdminTable columns={jobColumns} data={jobs} />
				{/if}
			</section>
		</div>
	{/if}
</div>

{#if selectedJob}
	<JobDetailsDialog job={selectedJob} bind:open={showJobDetails} />
{/if}
