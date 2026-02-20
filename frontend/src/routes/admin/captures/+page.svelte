<script lang="ts">
import { goto } from '$app/navigation';
import { page as pageStore } from '$app/state';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import Breadcrumb from '$lib/components/Breadcrumb.svelte';
import { DataTable, createDataTable } from '$lib/components/data-table';
import { Alert } from '$lib/components/ui/alert';
import { buttonVariants } from '$lib/components/ui/button';
import { NativeSelect } from '$lib/components/ui/native-select';
import { formatBytes } from '$lib/utils/format';
import { StatusBadge } from '$lib/components/ui/status-badge';
import type { PageData } from './$types';
import { columns } from './columns.js';

interface Props {
	data: PageData;
}
let { data }: Props = $props();
let captures = $derived(data.captures);
let totalCount = $derived(data.totalCount);
let currentPage = $derived(data.page);
let pageSize = $derived(data.pageSize);
let totalPages = $derived(Math.ceil(totalCount / pageSize));
let shaders = $derived(data.shaders);
let scenes = $derived(data.scenes);
let error = $derived(data.error);

const table = createDataTable<CaptureWithContext>({
	get data() {
		return captures;
	},
	columns,
	pageSize: false,
	selection: false,
	sorting: false
});

function pageUrl(p: number): string {
	const url = new URL(pageStore.url);
	url.searchParams.set('page', String(p));
	return url.toString();
}

function changePageSize(size: number) {
	const url = new URL(pageStore.url);
	url.searchParams.set('pageSize', String(size));
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}

const paginationBtnClass: string = buttonVariants({ variant: 'outline', size: 'sm' });

function setFilter(key: string, value: string) {
	const url = new URL(pageStore.url);
	if (value) {
		url.searchParams.set(key, value);
	} else {
		url.searchParams.delete(key);
	}
	url.searchParams.set('page', '1');
	void goto(url.toString(), { keepFocus: true });
}
</script>

<svelte:head><title>Captures - Glint</title></svelte:head>

<div class="space-y-4">
	<Breadcrumb segments={[{ label: 'Captures' }]} />

	<!-- Filter bar -->
	<div class="flex flex-wrap items-center gap-3">
		<NativeSelect size="sm" value={data.filters.shader ?? ''} onchange={(e) => setFilter('shader', e.currentTarget.value)}>
			<option value="">All shaders</option>
			{#each shaders as s (s.id)}
				<option value={s.slug}>{s.name}</option>
			{/each}
		</NativeSelect>

		<NativeSelect size="sm" value={data.filters.scene ?? ''} onchange={(e) => setFilter('scene', e.currentTarget.value)}>
			<option value="">All scenes</option>
			{#each scenes as s (s.id)}
				<option value={s.id}>{s.name}</option>
			{/each}
		</NativeSelect>

		<NativeSelect size="sm" value={data.filters.status ?? ''} onchange={(e) => setFilter('status', e.currentTarget.value)}>
			<option value="">All statuses</option>
			<option value="completed">Completed</option>
			<option value="failed">Failed</option>
		</NativeSelect>

		{#if data.filters.runId}
			<span class="inline-flex items-center gap-1 rounded-full bg-muted px-2.5 py-1 text-xs">
				Run: {data.filters.runId}
				<button class="hover:text-foreground" onclick={() => setFilter('runId', '')}>x</button
				>
			</span>
		{/if}
	</div>

	{#if error}
		<Alert variant="destructive">Error: {error}</Alert>
	{:else if captures.length === 0}
		<p class="text-muted-foreground">No captures yet.</p>
	{:else}
		<DataTable
			{table}
			getRowHref={(capture: CaptureWithContext) => `/admin/captures/${capture.id}`}
		>
			{#snippet card(capture: CaptureWithContext)}
				<div class="flex gap-3">
				{#if capture.image_path ?? capture.thumbhash}
					<CaptureImage
						src={capture.image_path}
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
							{capture.image_path ? 'No URL' : 'N/A'}
						</div>
					{/if}
					<div class="min-w-0 flex-1">
						<div class="font-medium">{capture.shader_name}</div>
						<div class="text-xs text-muted-foreground">
							{capture.shader_version}
							{#if capture.scene_name}
								&middot; {capture.scene_name}
							{/if}
						</div>
						<div class="mt-1 flex items-center gap-2">
							<StatusBadge status={capture.freshness}>{capture.freshness}</StatusBadge>
							{#if capture.resolution_width && capture.resolution_height}
								<span class="text-xs text-muted-foreground">
									{capture.resolution_width}x{capture.resolution_height}
								</span>
							{/if}
							{#if capture.file_size_bytes}
								<span class="text-xs text-muted-foreground">
									{formatBytes(capture.file_size_bytes)}
								</span>
							{/if}
						</div>
					</div>
					{#if capture.run_id && capture.run_status}
						<div class="shrink-0">
							<StatusBadge status={capture.run_status}>{capture.run_status}</StatusBadge>
						</div>
					{/if}
				</div>
			{/snippet}
		</DataTable>
	{/if}

	{#if totalPages > 1}
		<div class="flex items-center justify-between border-t pt-4">
			<div class="text-sm text-muted-foreground">
				Showing {(currentPage - 1) * pageSize + 1}&ndash;{Math.min(
					currentPage * pageSize,
					totalCount
				)} of {totalCount}
			</div>
			<div class="flex items-center gap-2">
				{#if currentPage > 1}
					<a
						href={pageUrl(currentPage - 1)}
						data-sveltekit-keepfocus
						class={paginationBtnClass}
					>
						Previous
					</a>
				{:else}
					<span class="{paginationBtnClass} pointer-events-none opacity-50">
						Previous
					</span>
				{/if}
				<span class="text-sm">Page {currentPage} of {totalPages}</span>
				{#if currentPage < totalPages}
					<a
						href={pageUrl(currentPage + 1)}
						data-sveltekit-keepfocus
						class={paginationBtnClass}
					>
						Next
					</a>
				{:else}
					<span class="{paginationBtnClass} pointer-events-none opacity-50">
						Next
					</span>
				{/if}
			</div>
			<NativeSelect size="sm" value={pageSize} onchange={(e) => changePageSize(Number(e.currentTarget.value))}>
				<option value={25}>25 per page</option>
				<option value={50}>50 per page</option>
				<option value={100}>100 per page</option>
			</NativeSelect>
		</div>
	{/if}
</div>
