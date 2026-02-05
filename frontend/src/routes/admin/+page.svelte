<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { invalidateAll } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import {
	RefreshCw,
	Play,
	Pause,
	Sparkles,
	Globe,
	Mountain,
	Camera,
	Briefcase,
	Users,
	ArrowRight
} from '@lucide/svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import type { CaptureWithContext } from '$lib/bindings';
import type { JobWithDetails } from '$lib/api/endpoints/admin';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();

let shaderCount: number = $derived(data.shaderCount);
let worldCount: number = $derived(data.worldCount);
let sceneCount: number = $derived(data.sceneCount);
let captureCount: number = $derived(data.captureCount);
let userCount: number = $derived(data.userCount);
let jobCounts: { pending: number; running: number; completed: number; failed: number } = $derived(
	data.jobCounts
);
let recentJobs: JobWithDetails[] = $derived(data.recentJobs);
let recentCaptures: CaptureWithContext[] = $derived(data.recentCaptures);
let healthStatus: 'ok' | 'error' = $derived(data.healthStatus);
let errors: Record<string, string> = $derived(data.errors);

let refreshing = $state(false);
let autoRefresh = $state(true);
let lastRefreshed = $state<Date | null>(null);

let refreshInterval: number | undefined;

interface StatCard {
	label: string;
	count: number;
	href: string;
	icon: typeof Sparkles;
}

const statCards = $derived<StatCard[]>([
	{ label: 'Shaders', count: shaderCount, href: '/admin/shaders', icon: Sparkles },
	{ label: 'Worlds', count: worldCount, href: '/admin/worlds', icon: Globe },
	{ label: 'Scenes', count: sceneCount, href: '/admin/scenes', icon: Mountain },
	{ label: 'Captures', count: captureCount, href: '/admin/captures', icon: Camera },
	{ label: 'Users', count: userCount, href: '/admin/users', icon: Users }
]);

async function refresh() {
	refreshing = true;
	await invalidateAll();
	lastRefreshed = new Date();
	refreshing = false;
}

function toggleAutoRefresh() {
	autoRefresh = !autoRefresh;
	if (autoRefresh) {
		refreshInterval = window.setInterval(() => {
			void refresh();
		}, 10000);
	} else if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = undefined;
	}
}

onMount(() => {
	lastRefreshed = new Date();
	if (autoRefresh) {
		refreshInterval = window.setInterval(() => {
			void refresh();
		}, 10000);
	}
});

onDestroy(() => {
	if (refreshInterval) {
		clearInterval(refreshInterval);
	}
});
</script>

<div class="space-y-6">
	<header class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<h1 class="text-2xl font-semibold">Dashboard</h1>
			<div
				class="flex h-3 w-3 items-center justify-center rounded-full"
				class:bg-green-500={healthStatus === 'ok'}
				class:bg-red-500={healthStatus === 'error'}
				title={healthStatus === 'ok'
					? 'API healthy'
					: `API error: ${errors.health || 'Unknown'}`}
			></div>
		</div>
		<div class="flex items-center gap-4">
			{#if lastRefreshed}
				<div class="text-sm text-muted-foreground">
					{#if refreshing}
						Refreshing...
					{:else}
						Updated <TimeAgo timestamp={lastRefreshed} />
					{/if}
				</div>
			{/if}
			<div class="flex gap-2">
				<Button
					variant={autoRefresh ? 'default' : 'outline'}
					size="icon"
					onclick={toggleAutoRefresh}
					title={autoRefresh ? 'Disable auto-refresh (10s)' : 'Enable auto-refresh (10s)'}
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
			</div>
		</div>
	</header>

	<!-- Stats Grid -->
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
			{#each statCards as card (card.label)}
				<a
					href={card.href}
					class="rounded-lg border bg-card p-4 transition-colors hover:bg-muted/50"
				>
					<div class="flex items-center justify-between">
						<card.icon class="h-5 w-5 text-muted-foreground" />
						<span class="text-2xl font-bold">{card.count}</span>
					</div>
					<div class="mt-2 text-sm text-muted-foreground">{card.label}</div>
				</a>
			{/each}
		</div>

		<!-- Jobs Summary -->
		<div class="rounded-lg border bg-card p-4">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="flex items-center gap-2 text-lg font-semibold">
					<Briefcase class="h-5 w-5" />
					Jobs
				</h2>
				<a href="/admin/jobs" class="flex items-center gap-1 text-sm text-primary hover:underline">
					View all <ArrowRight class="h-4 w-4" />
				</a>
			</div>
			<div class="grid gap-4 sm:grid-cols-4">
				<div class="rounded-md bg-yellow-50 p-3 dark:bg-yellow-900/20">
					<div class="text-2xl font-bold text-yellow-700 dark:text-yellow-300">
						{jobCounts.pending}
					</div>
					<div class="text-sm text-yellow-600 dark:text-yellow-400">Pending</div>
				</div>
				<div class="rounded-md bg-blue-50 p-3 dark:bg-blue-900/20">
					<div class="text-2xl font-bold text-blue-700 dark:text-blue-300">
						{jobCounts.running}
					</div>
					<div class="text-sm text-blue-600 dark:text-blue-400">Running</div>
				</div>
				<div class="rounded-md bg-green-50 p-3 dark:bg-green-900/20">
					<div class="text-2xl font-bold text-green-700 dark:text-green-300">
						{jobCounts.completed}
					</div>
					<div class="text-sm text-green-600 dark:text-green-400">Completed</div>
				</div>
				<div class="rounded-md bg-red-50 p-3 dark:bg-red-900/20">
					<div class="text-2xl font-bold text-red-700 dark:text-red-300">
						{jobCounts.failed}
					</div>
					<div class="text-sm text-red-600 dark:text-red-400">Failed</div>
				</div>
			</div>
		</div>

		<!-- Recent Activity Grid -->
		<div class="grid gap-4 lg:grid-cols-2">
			<!-- Recent Jobs -->
			<div class="rounded-lg border bg-card p-4">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Recent Jobs</h2>
					<a
						href="/admin/jobs"
						class="flex items-center gap-1 text-sm text-primary hover:underline"
					>
						View all <ArrowRight class="h-4 w-4" />
					</a>
				</div>
				{#if recentJobs.length === 0}
					<p class="text-sm text-muted-foreground">No jobs yet</p>
				{:else}
					<div class="space-y-2">
						{#each recentJobs as job (job.id)}
							{@const statusColor =
								{
									pending: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
									claimed: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
									running: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
									completed: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
									failed: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
								}[job.status] ?? 'bg-gray-100 text-gray-800'}
							<div class="flex items-center justify-between rounded border p-2 text-sm">
								<div>
									<div class="font-medium">{job.shader_name}</div>
									<div class="text-xs text-muted-foreground">
										{job.scene_count} scenes &middot; {job.shader_version}
									</div>
								</div>
								<span class="rounded px-2 py-0.5 text-xs font-medium {statusColor}">
									{job.status}
								</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Recent Captures -->
			<div class="rounded-lg border bg-card p-4">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Recent Captures</h2>
					<a
						href="/admin/captures"
						class="flex items-center gap-1 text-sm text-primary hover:underline"
					>
						View all <ArrowRight class="h-4 w-4" />
					</a>
				</div>
				{#if recentCaptures.length === 0}
					<p class="text-sm text-muted-foreground">No captures yet</p>
				{:else}
					<div class="space-y-2">
						{#each recentCaptures as capture (capture.id)}
							<div class="flex items-center gap-3 rounded border p-2 text-sm">
								{#if capture.screenshot_url}
									<img
										src={capture.screenshot_url}
										alt="Capture"
										class="h-10 w-16 rounded object-cover"
									/>
								{:else}
									<div class="flex h-10 w-16 items-center justify-center rounded bg-muted text-xs text-muted-foreground">
										N/A
									</div>
								{/if}
								<div class="min-w-0 flex-1">
									<div class="truncate font-medium">{capture.shader_name}</div>
									<div class="text-xs text-muted-foreground">
										{capture.shader_version}
										{#if capture.profile}
											&middot; {capture.profile}
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
</div>
