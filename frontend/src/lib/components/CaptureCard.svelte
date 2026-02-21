<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { StatusBadge, type StatusBadgeStatus } from '$lib/components/ui/status-badge';
import { formatBytes } from '$lib/utils/format';

interface CaptureData {
	imagePath: string | null;
	thumbhash: string | null;
	freshness: string;
	resolutionWidth?: number | null;
	resolutionHeight?: number | null;
	fileSizeBytes?: number | null;
	runId?: unknown;
	runStatus?: string | null;
}

interface Props {
	capture: CaptureData;
	title: string;
	subtitle?: string;
	layout: 'row' | 'tile';
}

let { capture, title, subtitle, layout }: Props = $props();
</script>

{#if layout === 'row'}
	<div class="flex gap-3">
		{#if capture.imagePath ?? capture.thumbhash}
			<CaptureImage
				src={capture.imagePath}
				thumbhash={capture.thumbhash}
				preset="thumbnail"
				alt="Capture preview"
				class="h-full w-full object-cover"
				containerClass="h-12 w-20 shrink-0 rounded"
			/>
		{:else}
			<div
				class="flex h-12 w-20 shrink-0 items-center justify-center rounded bg-muted text-xs text-muted-foreground"
			>
				N/A
			</div>
		{/if}
		<div class="min-w-0 flex-1">
			<div class="font-medium">{title}</div>
			{#if subtitle}
				<div class="text-xs text-muted-foreground">{subtitle}</div>
			{/if}
			<div class="mt-1 flex items-center gap-2">
				<StatusBadge status={capture.freshness.toLowerCase() as StatusBadgeStatus}>{capture.freshness}</StatusBadge>
				{#if capture.resolutionWidth && capture.resolutionHeight}
					<span class="text-xs text-muted-foreground">
						{capture.resolutionWidth}x{capture.resolutionHeight}
					</span>
				{/if}
				{#if capture.fileSizeBytes}
					<span class="text-xs text-muted-foreground">
						{formatBytes(capture.fileSizeBytes)}
					</span>
				{/if}
			</div>
		</div>
		{#if capture.runId && capture.runStatus}
			<div class="shrink-0">
				<StatusBadge status={capture.runStatus.toLowerCase() as StatusBadgeStatus}>{capture.runStatus}</StatusBadge>
			</div>
		{/if}
	</div>
{:else}
	{#if capture.imagePath ?? capture.thumbhash}
		<CaptureImage
			src={capture.imagePath}
			thumbhash={capture.thumbhash}
			preset="card"
			alt="Capture preview"
			class="h-full w-full object-cover"
			containerClass="aspect-video w-full"
		/>
	{:else}
		<div class="flex aspect-video w-full items-center justify-center bg-muted text-xs text-muted-foreground">
			No image
		</div>
	{/if}
	<div class="space-y-1 p-3">
		<div class="truncate text-sm font-medium">{title}</div>
		{#if subtitle}
			<div class="truncate text-xs text-muted-foreground">{subtitle}</div>
		{/if}
		<div class="flex items-center gap-2">
			<StatusBadge status={capture.freshness.toLowerCase() as StatusBadgeStatus}>{capture.freshness}</StatusBadge>
			{#if capture.resolutionWidth && capture.resolutionHeight}
				<span class="text-xs text-muted-foreground">
					{capture.resolutionWidth}x{capture.resolutionHeight}
				</span>
			{/if}
			{#if capture.fileSizeBytes}
				<span class="text-xs text-muted-foreground">
					{formatBytes(capture.fileSizeBytes)}
				</span>
			{/if}
		</div>
	</div>
{/if}
