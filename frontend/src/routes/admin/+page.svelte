<script lang="ts">
import { invalidateAll } from '$app/navigation';
import type { CaptureHealthSummary, CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ItemGrid } from '$lib/components/item-grid';
import ErrorBanner from '$lib/components/ErrorBanner.svelte';
import RefreshButton from '$lib/components/RefreshButton.svelte';
import SectionBoundary from '$lib/components/SectionBoundary.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { Button } from '$lib/components/ui/button';
import { formatBytes, formatDatetime } from '$lib/utils/format';
import {
	Activity,
	ArrowRight,
	Camera,
	Globe,
	HardDrive,
	HeartPulse,
	Mountain,
	Pause,
	Play,
	Sparkles,
	Users
} from '@lucide/svelte';
import { scaleLinear, scaleTime } from 'd3-scale';
import {
	Area,
	Axis,
	Chart,
	Highlight,
	LinearGradient,
	RectClipPath,
	Svg,
	Tooltip
} from 'layerchart';
import { onDestroy, onMount } from 'svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

let shaderCount: number = $derived(data.shaderCount);
let worldCount: number = $derived(data.worldCount);
let sceneCount: number = $derived(data.sceneCount);
let captureCount: number = $derived(data.captureCount);
let userCount: number = $derived(data.userCount);
let runCount: number = $derived(data.runCount);
let recentCaptures: CaptureWithContext[] = $derived(data.recentCaptures);
let healthStatus: 'ok' | 'error' = $derived(data.healthStatus);
let captureHealth: CaptureHealthSummary | null = $derived(data.captureHealth);
let storageStats = $derived(data.storageStats);
let storageGrowth = $derived(data.storageGrowth);
let chartData = $derived(
	storageGrowth.map(
		(b: {
			date: number;
			cumulative_bytes: number;
			cumulative_count: number;
			bucket_bytes: number;
		}) => ({
			date: new Date(b.date * 1000),
			bytes: b.cumulative_bytes,
			count: b.cumulative_count,
			bucketBytes: b.bucket_bytes
		})
	)
);
let errors: Record<string, string> = $derived(data.errors);
const failedSections = $derived(Object.keys(errors).filter((k) => k !== 'health'));

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
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
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

<svelte:head><title>Admin - Glint</title></svelte:head>

<div class="space-y-6">
	<header class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<h1 class="text-2xl font-semibold">Dashboard</h1>
			<div
				class="flex h-3 w-3 items-center justify-center rounded-full"
			class:bg-success={healthStatus === 'ok'}
			class:bg-destructive={healthStatus === 'error'}
				title={healthStatus === 'ok'
					? 'API healthy'
					: `API error: ${errors.health || 'Unknown'}`}
			></div>
		</div>
		<div class="flex items-center gap-4">
			{#if lastRefreshed}
			<div class="text-sm text-foreground">
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
				<RefreshButton {refreshing} onclick={refresh} />
			</div>
		</div>
	</header>

	<!-- Error Summary -->
	{#if failedSections.length > 0}
		<ErrorBanner message="Failed to load: {failedSections.join(', ')}" />
	{/if}

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

		<!-- Capture Health -->
		{#if captureHealth}
			<a
				href="/admin/health"
				class="flex items-center gap-4 rounded-lg border bg-card p-4 transition-colors hover:bg-muted/50"
			>
				<HeartPulse class="h-5 w-5 text-muted-foreground" />
				<div class="flex flex-1 items-center gap-3 text-sm">
					<span class="font-medium">Capture Health</span>
				<span class="text-success">{captureHealth.completed} completed</span>
				{#if captureHealth.missing > 0}
					<span class="text-destructive">{captureHealth.missing} missing</span>
				{/if}
				{#if captureHealth.stale > 0}
					<span class="text-warning">{captureHealth.stale} stale</span>
				{/if}
					{#if captureHealth.failed > 0}
						<span class="text-muted-foreground">{captureHealth.failed} failed</span>
					{/if}
				</div>
				<ArrowRight class="h-4 w-4 text-muted-foreground" />
			</a>
		{/if}

	<!-- Storage Stats -->
	{#if storageStats}
		<div class="rounded-lg border bg-card p-4">
			<div class="mb-3 flex items-center gap-2">
				<HardDrive class="h-5 w-5 text-muted-foreground" />
				<span class="font-medium">Storage</span>
			</div>
			<div class="grid grid-cols-4 gap-4 text-sm">
				<div>
					<div class="text-2xl font-bold">{formatBytes(storageStats.total_bytes)}</div>
					<div class="text-muted-foreground">Total</div>
				</div>
				<div>
					<div class="text-2xl font-bold">{storageStats.capture_count}</div>
					<div class="text-muted-foreground">Tracked</div>
				</div>
				<div>
					<div class="text-2xl font-bold">{formatBytes(storageStats.avg_bytes)}</div>
					<div class="text-muted-foreground">Average</div>
				</div>
				<div>
					<div class="text-2xl font-bold">{storageStats.missing_count}</div>
					<div class="text-muted-foreground">Missing</div>
				</div>
			</div>
		{#if chartData.length > 1}
			<SectionBoundary title="Storage chart">
			<div class="mt-2 text-xs text-muted-foreground">Storage Growth</div>
			<div class="mt-1 h-75">
				<Chart
					data={chartData}
					x="date"
					xScale={scaleTime()}
					y="bytes"
					yScale={scaleLinear()}
					yNice={true}
					yDomain={[0, null]}
					padding={{ top: 48, bottom: 24, left: 60 }}
					tooltip={{ mode: 'bisect-x' }}
					let:width
					let:height
					let:padding
					let:tooltip
				>
					{@const tt = tooltip as { data: unknown; x: number }}
					{@const pad = padding as { top: number; left: number }}
					<Svg>
						<LinearGradient
							stops={[
								'color-mix(in oklab, var(--chart-3) 50%, transparent)',
								'transparent'
							]}
							vertical
							let:gradient
						>
							<Area
								line={{ class: 'stroke-2 stroke-chart-3 opacity-20' }}
								fill={gradient}
							/>
							<RectClipPath
								x={0}
								y={0}
								width={tt.data ? tt.x - pad.left : width}
								{height}
								spring
							>
								<Area line={{ class: 'stroke-2 stroke-chart-3' }} fill={gradient} />
							</RectClipPath>
						</LinearGradient>
						<Highlight
							points
							lines={{ class: 'stroke-chart-3 [stroke-dasharray:unset]' }}
						/>
						<Axis placement="bottom" />
						<Axis
							placement="left"
							ticks={4}
							format={(v: number) => formatBytes(v)}
						/>
					</Svg>

					<!-- Floating label: total size near the data point -->
					<Tooltip.Root
						y={48}
						xOffset={4}
						variant="none"
						class="text-sm font-semibold text-primary leading-3"
						let:data
					>
						{@const d = data as { bytes: number; count: number }}
						{formatBytes(d.bytes)} &middot; {d.count} captures
					</Tooltip.Root>

					<!-- Floating label: date at top-left corner -->
					<Tooltip.Root
						x={4}
						y={4}
						variant="none"
						class="text-sm font-semibold leading-3"
						let:data
					>
						{@const d = data as { date: Date }}
						{formatDatetime(d.date)}
					</Tooltip.Root>

					<!-- Floating label: date pill on bottom axis -->
					<Tooltip.Root
						x="data"
						y={height + pad.top + 2}
						anchor="top"
						variant="none"
						class="text-sm font-semibold bg-primary text-primary-foreground leading-3 px-2 py-1 rounded whitespace-nowrap"
						let:data
					>
						{@const d = data as { date: Date }}
						{formatDatetime(d.date)}
					</Tooltip.Root>
				</Chart>
			</div>
		</SectionBoundary>
		{/if}
		</div>
	{/if}

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
			<ItemGrid items={recentCaptures} key={(c: CaptureWithContext) => c.id} size="small">
				{#snippet card(capture: CaptureWithContext)}
					<a href="/admin/captures/{capture.id}" class="overflow-hidden rounded-lg border transition-colors hover:bg-muted/50">
						{#if capture.image_url}
							<CaptureImage
								src={capture.image_url}
								thumbhash={capture.thumbhash}
								preset="card"
								alt={capture.shader_name}
								class="w-full"
								containerClass="aspect-video w-full"
							/>
						{:else}
							<div class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground">
								No image
							</div>
						{/if}
						<div class="space-y-1 p-3">
							<div class="flex items-center justify-between gap-2">
								<span class="truncate font-medium text-sm">{capture.shader_name}</span>
								{#if capture.scene_name}
									<span class="shrink-0 rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">{capture.scene_name}</span>
								{/if}
							</div>
							<div class="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-muted-foreground">
								<span>{capture.shader_version}</span>
							{#if capture.profile_name}
								<span>&middot; {capture.profile_name}</span>
							{/if}
								{#if capture.resolution_width && capture.resolution_height}
									<span>&middot; {capture.resolution_width}&times;{capture.resolution_height}</span>
								{/if}
								{#if capture.file_size_bytes}
									<span>&middot; {formatBytes(capture.file_size_bytes)}</span>
								{/if}
							</div>
							{#if capture.captured_at}
								<div class="text-xs text-muted-foreground">
									<TimeAgo timestamp={capture.captured_at} />
								</div>
							{/if}
						</div>
					</a>
				{/snippet}
			</ItemGrid>
			{/if}
		</div>
</div>
