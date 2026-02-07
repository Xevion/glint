<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { Button } from '$lib/components/ui/button';
import {
	Activity,
	ArrowRight,
	Camera,
	Globe,
	Mountain,
	Pause,
	Play,
	RefreshCw,
	Sparkles,
	Users
} from '@lucide/svelte';
import { onDestroy, onMount } from 'svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();

let shaderCount: number = $derived(data.shaderCount);
let worldCount: number = $derived(data.worldCount);
let sceneCount: number = $derived(data.sceneCount);
let captureCount: number = $derived(data.captureCount);
let userCount: number = $derived(data.userCount);
let runCount: number = $derived(data.runCount);
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
	{ label: 'Runs', count: runCount, href: '/admin/runs', icon: Activity },
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
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-6">
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
							{#if capture.image_url}
								<CaptureImage
									src={capture.image_url}
									thumbhash={capture.thumbhash}
									preset="thumbnail"
									alt="Capture"
									class="h-full w-full object-cover"
									containerClass="h-10 w-16 rounded"
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
