<script lang="ts">
import { goto } from '$app/navigation';
import { api } from '$lib/api';
import type { CaptureWithContext } from '$lib/bindings';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import TimeAgo from '$lib/components/TimeAgo.svelte';
import { AdminDetailField } from '$lib/components/admin';
import { Button } from '$lib/components/ui/button';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import { ArrowLeft, Trash2 } from '@lucide/svelte';
import type { PageData } from './$types';

let { data } = $props<{ data: PageData }>();
let capture: CaptureWithContext = $derived(data.capture);

let error = $state<string | null>(null);
let actionLoading = $state(false);
let showDeleteConfirm = $state(false);

const statusColors: Record<string, string> = {
	running: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
	completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
	partial: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
	failed: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
};

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
</script>

<div class="space-y-6">
	<!-- Header -->
	<header class="space-y-2">
		<div class="flex items-center gap-2">
		<a href="/admin/captures" class="text-muted-foreground hover:text-foreground" aria-label="Back to captures">
			<ArrowLeft class="h-4 w-4" />
		</a>
			<h1 class="text-2xl font-semibold">
				{capture.shader_name} - {capture.shader_version}
			</h1>
			{#if capture.profile}
				<span class="rounded-full bg-muted px-2 py-0.5 text-xs font-medium">
					{capture.profile}
				</span>
			{/if}
		</div>
	</header>

	{#if error}
		<div
			class="rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
		>
			{error}
		</div>
	{/if}

	<!-- Image Hero -->
	{#if capture.image_url}
		<CaptureImage
			src={capture.image_url}
			thumbhash={capture.thumbhash}
			preset="hero"
			alt="Capture"
			lightbox
			class="w-full"
			containerClass="w-full overflow-hidden rounded-lg border"
		/>
	{/if}

	<!-- Metadata -->
	<div class="rounded-lg border p-4">
		<dl class="space-y-2 text-sm">
			<AdminDetailField label="ID">
				<code class="text-xs">{capture.id}</code>
			</AdminDetailField>

			<AdminDetailField label="Shader">
				<a
					href="/admin/shaders/{capture.shader_slug}"
					class="text-primary hover:underline"
				>
					{capture.shader_name}
				</a>
				<span class="text-muted-foreground"> ({capture.shader_version})</span>
			</AdminDetailField>

			<AdminDetailField label="Scene">
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
			</AdminDetailField>

			{#if capture.profile}
				<AdminDetailField label="Profile">
					{capture.profile}
				</AdminDetailField>
			{/if}

			<AdminDetailField label="Resolution">
				{capture.resolution_width && capture.resolution_height
					? `${capture.resolution_width}x${capture.resolution_height}`
					: '-'}
			</AdminDetailField>

			<AdminDetailField label="Captured">
				{#if capture.captured_at}
					<TimeAgo timestamp={capture.captured_at} />
				{:else}
					-
				{/if}
			</AdminDetailField>

			{#if capture.run_id}
				<AdminDetailField label="Run">
					<a
						href="/admin/runs/{capture.run_id}"
						class="text-primary hover:underline"
					>
						{capture.run_id.slice(0, 12)}...
					</a>
					{#if capture.run_status}
						<span
							class="ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {statusColors[
								capture.run_status
							] ?? ''}"
						>
							{capture.run_status}
						</span>
					{/if}
				</AdminDetailField>
			{/if}
		</dl>
	</div>

	<!-- Actions -->
	<div class="border-t pt-4">
		<Button variant="destructive" onclick={() => (showDeleteConfirm = true)} disabled={actionLoading}>
			<Trash2 class="mr-2 h-4 w-4" />
			Delete Capture
		</Button>
	</div>
</div>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Capture"
	description={`Delete capture for ${capture.shader_name}?`}
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
