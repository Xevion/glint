<script lang="ts">
import { goto } from '$app/navigation';
import { api } from '$lib/api';
import type { CaptureDetail, CaptureWithContext } from '$lib/bindings';
import CaptureGridAdmin from '$lib/components/CaptureGridAdmin.svelte';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import Lightbox from '$lib/components/Lightbox.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminBreadcrumb } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Tabs from '$lib/components/ui/tabs';
import { formatBytes } from '$lib/utils/format';
import { preloadImage } from '$lib/utils/image';
import { freshnessColors, statusColorFallback, statusColors } from '$lib/utils/status';
import { Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let capture: CaptureDetail = $derived(data.capture);

let error = $state<string | null>(null);
let actionLoading = $state(false);
let showDeleteConfirm = $state(false);
let lightboxOpen = $state(false);

// Build breadcrumb segments
let breadcrumbs = $derived.by(() => {
	const segments: { label: string; href?: string }[] = [
		{ label: 'Captures', href: '/admin/captures' },
		{ label: capture.shader_name, href: `/admin/shaders/${capture.shader_slug}` }
	];
	if (capture.profile) {
		segments.push({ label: capture.profile });
	}
	if (capture.scene_name) {
		segments.push({
			label: capture.scene_name,
			href: `/admin/scenes/${capture.scene_id}`
		});
	} else {
		segments.push({ label: capture.scene_id });
	}
	return segments;
});

// Technical fields: only show section if at least one is non-null
let hasTechnicalData = $derived(
	capture.avg_fps != null ||
		capture.min_fps != null ||
		capture.max_fps != null ||
		capture.frame_time_avg != null ||
		capture.frame_time_p99 != null ||
		capture.minecraft_version != null ||
		capture.iris_version != null ||
		capture.gpu_model != null
);

// Tab configuration
let hasRelatedCaptures = $derived(
	capture.same_shader_scene.length > 0 ||
		capture.same_scene.length > 0 ||
		capture.same_run.length > 0
);

let activeTab = $state('');

// Set initial tab when capture changes
$effect(() => {
	void capture.id;
	activeTab =
		capture.same_shader_scene.length > 0
			? 'shader-scene'
			: capture.same_scene.length > 0
				? 'scene'
				: 'run';
});

let viewAllHref = $derived.by(() => {
	const counts: Record<string, number> = {
		'shader-scene': capture.same_shader_scene.length,
		scene: capture.same_scene.length,
		run: capture.same_run.length
	};
	if (counts[activeTab] < 8) return null;

	switch (activeTab) {
		case 'shader-scene':
			return `/admin/captures?scene=${capture.scene_id}&shader=${capture.shader_slug}`;
		case 'scene':
			return `/admin/captures?scene=${capture.scene_id}`;
		case 'run':
			return `/admin/captures?runId=${capture.run_id}`;
		default:
			return null;
	}
});

async function confirmDelete() {
	actionLoading = true;
	try {
		const result = await api.admin.deleteCapture(capture.id);
		result.match({
			Ok: () => {
				void goto('/admin/captures');
			},
			Err: (err) => {
				error = err.message;
			}
		});
	} finally {
		actionLoading = false;
	}
}

function formatFps(value: number | null): string {
	if (value == null) return '\u2014';
	return value.toFixed(1);
}

function formatMs(value: number | null): string {
	if (value == null) return '\u2014';
	return `${value.toFixed(2)}ms`;
}
</script>

<svelte:head><title>Capture Details - Glint</title></svelte:head>

<div class="space-y-6">
	<!-- Breadcrumb Header -->
	<header class="flex items-center justify-between">
		<AdminBreadcrumb
			backHref="/admin/captures"
			backLabel="Back to captures"
			segments={breadcrumbs}
		>
			{#snippet trailing()}
				{#if capture.status !== 'completed'}
					<span
						class="ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
							capture.status
						] ?? statusColorFallback}"
					>
						{capture.status}
					</span>
				{/if}
				<span
					class="ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {freshnessColors[capture.freshness]}"
				>
					{capture.freshness}
				</span>
			{/snippet}
		</AdminBreadcrumb>
		<Button
			variant="destructive"
			size="icon"
			onclick={() => (showDeleteConfirm = true)}
			disabled={actionLoading}
			aria-label="Delete capture"
		>
			<Trash2 class="h-4 w-4" />
		</Button>
	</header>

	{#if error}
		<Alert variant="destructive">{error}</Alert>
	{/if}

	<!-- Image + Metadata Split -->
	<div class="grid gap-6 lg:grid-cols-[3fr_2fr]">
		<!-- Hero Image -->
		{#if capture.image_url ?? capture.thumbhash}
			<button
				type="button"
				class="group relative w-full cursor-pointer overflow-hidden rounded-lg border"
				onclick={() => (lightboxOpen = true)}
				onmouseenter={() => preloadImage(capture.image_url, 'full')}
				disabled={!capture.image_url}
			>
				<CaptureImage
					src={capture.image_url}
					thumbhash={capture.thumbhash}
					aspectRatio={capture.resolution_width && capture.resolution_height
						? capture.resolution_width / capture.resolution_height
						: 16 / 9}
					preset="hero"
					alt="Capture"
					class="w-full transition-transform duration-300 group-hover:scale-[1.02]"
					containerClass="w-full"
				/>
			</button>
		{:else}
			<div
				class="flex aspect-video w-full items-center justify-center rounded-lg border bg-muted text-sm text-muted-foreground"
			>
				No image
			</div>
		{/if}

		<!-- Metadata Sidebar -->
		<div class="space-y-4">
			<!-- Capture Info -->
			<div class="rounded-lg border bg-card p-4">
				<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
					<dt class="text-muted-foreground">ID</dt>
					<dd><code class="text-xs">{capture.id}</code></dd>

					<dt class="text-muted-foreground">Shader</dt>
					<dd>
						<a
							href="/admin/shaders/{capture.shader_slug}"
							class="text-primary hover:underline"
						>
							{capture.shader_name}
						</a>
						<span class="text-muted-foreground"> {capture.shader_version}</span>
					</dd>

					<dt class="text-muted-foreground">Scene</dt>
					<dd>
						{#if capture.scene_name}
							<a
								href="/admin/scenes/{capture.scene_id}"
								class="text-primary hover:underline"
							>
								{capture.scene_name}
							</a>
						{:else}
							<code class="text-xs">{capture.scene_id}</code>
						{/if}
					</dd>

					{#if capture.profile}
						<dt class="text-muted-foreground">Profile</dt>
						<dd>{capture.profile}</dd>
					{/if}

					<dt class="text-muted-foreground">Resolution</dt>
					<dd>
						{capture.resolution_width && capture.resolution_height
							? `${capture.resolution_width}\u00d7${capture.resolution_height}`
							: '\u2014'}
					</dd>

					{#if capture.file_size_bytes != null}
						<dt class="text-muted-foreground">File size</dt>
						<dd>{formatBytes(capture.file_size_bytes)}</dd>
					{/if}

					{#if capture.content_type}
						<dt class="text-muted-foreground">Type</dt>
						<dd>{capture.content_type}</dd>
					{/if}

				<dt class="text-muted-foreground">Captured</dt>
				<dd>
					{#if capture.captured_at}
						<TimeAgo timestamp={capture.captured_at} />
					{:else}
						&mdash;
					{/if}
				</dd>

				<dt class="text-muted-foreground">Freshness</dt>
				<dd>
					<span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {freshnessColors[capture.freshness]}">
						{capture.freshness}
					</span>
				</dd>

				{#if capture.run_id}
						<dt class="text-muted-foreground">Run</dt>
						<dd>
							<a
								href="/admin/runs/{capture.run_id}"
								class="text-primary hover:underline"
							>
								{capture.run_id}
							</a>
							{#if capture.run_status}
								<span
									class="ml-1 inline-flex items-center rounded-full px-1.5 py-0.5 text-xs font-medium {statusColors[
										capture.run_status
									] ?? statusColorFallback}"
								>
									{capture.run_status}
								</span>
							{/if}
						</dd>
					{/if}
				</dl>
			</div>

			<!-- Error Message -->
			{#if capture.error_message}
				<div class="rounded-lg border border-destructive/50 bg-destructive/5 p-4">
					<div class="text-xs font-medium text-destructive">Error</div>
					<p class="mt-1 text-sm">{capture.error_message}</p>
				</div>
			{/if}

			<!-- Technical Metadata -->
			{#if hasTechnicalData}
				<div class="rounded-lg border bg-card p-4">
					<h3 class="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
						Technical
					</h3>
					<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
						{#if capture.avg_fps != null || capture.min_fps != null || capture.max_fps != null}
							<dt class="text-muted-foreground">FPS</dt>
							<dd>
								{formatFps(capture.avg_fps)} avg
								<span class="text-muted-foreground">
									({formatFps(capture.min_fps)}&ndash;{formatFps(capture.max_fps)})
								</span>
							</dd>
						{/if}

						{#if capture.frame_time_avg != null || capture.frame_time_p99 != null}
							<dt class="text-muted-foreground">Frame time</dt>
							<dd>
								{formatMs(capture.frame_time_avg)} avg
								<span class="text-muted-foreground">
									(p99: {formatMs(capture.frame_time_p99)})
								</span>
							</dd>
						{/if}

						{#if capture.minecraft_version}
							<dt class="text-muted-foreground">Minecraft</dt>
							<dd>{capture.minecraft_version}</dd>
						{/if}

						{#if capture.iris_version}
							<dt class="text-muted-foreground">Iris</dt>
							<dd>{capture.iris_version}</dd>
						{/if}

						{#if capture.gpu_model}
							<dt class="text-muted-foreground">GPU</dt>
							<dd>{capture.gpu_model}</dd>
						{/if}
					</dl>
				</div>
			{/if}
		</div>
	</div>

	<!-- Related Captures -->
	{#if hasRelatedCaptures}
		<Tabs.Root bind:value={activeTab}>
			<div class="flex items-center gap-3">
				<Tabs.List>
					{#if capture.same_shader_scene.length > 0}
					<Tabs.Trigger value="shader-scene">
						Same Shader + Scene ({capture.same_shader_scene.length >= 8 ? '8+' : capture.same_shader_scene.length})
					</Tabs.Trigger>
				{/if}
				{#if capture.same_scene.length > 0}
					<Tabs.Trigger value="scene">
						Same Scene ({capture.same_scene.length >= 8 ? '8+' : capture.same_scene.length})
					</Tabs.Trigger>
				{/if}
				{#if capture.same_run.length > 0}
					<Tabs.Trigger value="run">
						Same Run ({capture.same_run.length >= 8 ? '8+' : capture.same_run.length})
					</Tabs.Trigger>
					{/if}
				</Tabs.List>
				{#if viewAllHref}
					<a href={viewAllHref} class="text-sm text-primary hover:underline">
						View all &rarr;
					</a>
				{/if}
			</div>

			{#if capture.same_shader_scene.length > 0}
				<Tabs.Content value="shader-scene">
					<CaptureGridAdmin
						captures={capture.same_shader_scene}
						alt={(c: CaptureWithContext) => `${c.shader_name} ${c.shader_version}`}
					>
					{#snippet footer(c: CaptureWithContext)}
						<div class="p-2">
							<div class="flex items-center justify-between">
								<div class="text-sm font-medium">
									{c.shader_version}
									{#if c.profile}
										&middot; {c.profile}
									{/if}
								</div>
								{#if c.freshness !== 'fresh'}
								<span class="inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium {freshnessColors[c.freshness]}">
										{c.freshness}
									</span>
								{/if}
							</div>
							{#if c.captured_at}
								<div class="text-xs text-muted-foreground">
									<TimeAgo timestamp={c.captured_at} />
								</div>
							{/if}
						</div>
					{/snippet}
				</CaptureGridAdmin>
			</Tabs.Content>
		{/if}

		{#if capture.same_scene.length > 0}
				<Tabs.Content value="scene">
					<CaptureGridAdmin
						captures={capture.same_scene}
						alt={(c: CaptureWithContext) => `${c.shader_name} in ${c.scene_name ?? 'scene'}`}
					>
						{#snippet footer(c: CaptureWithContext)}
							<div class="p-2">
								<div class="text-sm font-medium">{c.shader_name}</div>
								<div class="text-xs text-muted-foreground">
									{c.shader_version}
									{#if c.profile}
										&middot; {c.profile}
									{/if}
								</div>
							</div>
						{/snippet}
					</CaptureGridAdmin>
				</Tabs.Content>
			{/if}

			{#if capture.same_run.length > 0}
				<Tabs.Content value="run">
					<CaptureGridAdmin
						captures={capture.same_run}
						alt={(c: CaptureWithContext) => `${c.shader_name} - ${c.scene_name ?? 'scene'}`}
					>
						{#snippet footer(c: CaptureWithContext)}
							<div class="p-2">
								<div class="text-sm font-medium">{c.shader_name}</div>
								<div class="text-xs text-muted-foreground">
									{c.scene_name ?? c.scene_id}
									{#if c.profile}
										&middot; {c.profile}
									{/if}
								</div>
							</div>
						{/snippet}
					</CaptureGridAdmin>
				</Tabs.Content>
			{/if}
		</Tabs.Root>
	{/if}
</div>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Capture"
	description={`Delete capture for ${capture.shader_name}?`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>

{#if lightboxOpen && capture.image_url}
	<Lightbox
		captures={[capture]}
		currentIndex={0}
		onClose={() => (lightboxOpen = false)}
		onNavigate={() => { /* single capture, no navigation */ }}
	/>
{/if}
