<svelte:head><title>Settings - Glint</title></svelte:head>

<script lang="ts">
import { invalidateAll } from '$app/navigation';
import { createApiClient } from '$lib/api';
import type { Background, ThemeMode } from '$lib/bindings';
import { AdminPageHeader } from '$lib/components/admin';
import { Alert } from '$lib/components/ui/alert';
import { ConfirmDialog } from '$lib/components/ui/dialog';
import * as Select from '$lib/components/ui/select';
import { cfImageUrl } from '$lib/utils/image';
import { formatBytes } from '$lib/utils/format';
import { decodeThumbhash } from '$lib/utils/thumbhash';
import { ImagePlus, Trash2, Upload, Eye, EyeOff } from '@lucide/svelte';
import { rgbaToThumbHash } from 'thumbhash';
import type { PageData } from './$types';

let { data }: { data: PageData } = $props();
let backgrounds = $derived(data.backgrounds);
let error = $derived(data.error);

let refreshing = $state(false);
let uploading = $state(false);
let uploadError = $state<string | null>(null);
let dragOver = $state(false);

let deleteTarget = $state<Background | null>(null);
let showDeleteConfirm = $state(false);
let actionLoading = $state(false);

const api = createApiClient();

async function refresh() {
	refreshing = true;
	await Promise.all([invalidateAll(), new Promise((r) => setTimeout(r, 300))]);
	refreshing = false;
}

// -- Upload flow --

function handleDragOver(e: DragEvent) {
	e.preventDefault();
	dragOver = true;
}

function handleDragLeave() {
	dragOver = false;
}

function handleDrop(e: DragEvent) {
	e.preventDefault();
	dragOver = false;
	const files = e.dataTransfer?.files;
	if (files) void uploadFiles(files);
}

function handleFileSelect(e: Event) {
	const input = e.target as HTMLInputElement;
	if (input.files) void uploadFiles(input.files);
	input.value = '';
}

async function uploadFiles(files: FileList) {
	const imageFiles = Array.from(files).filter((f) => f.type.startsWith('image/'));
	if (imageFiles.length === 0) {
		uploadError = 'No image files selected';
		return;
	}

	uploading = true;
	uploadError = null;

	for (const file of imageFiles) {
		try {
			await uploadSingleFile(file);
		} catch (err) {
			uploadError = err instanceof Error ? err.message : 'Upload failed';
			break;
		}
	}

	uploading = false;
	await refresh();
}

async function uploadSingleFile(file: File) {
	// 1. Initiate upload to get presigned URL
	const initiateResult = await api.backgrounds.initiateUpload(file.name, file.type);
	if (initiateResult.isErr) {
		throw new Error(initiateResult.error.message);
	}
	const { id, presigned_url } = initiateResult.value;

	// 2. Upload file directly to R2 via presigned URL
	const uploadResponse = await fetch(presigned_url, {
		method: 'PUT',
		body: file,
		headers: { 'Content-Type': file.type }
	});

	if (!uploadResponse.ok) {
		throw new Error(`R2 upload failed: ${uploadResponse.status}`);
	}

	// 3. Compute thumbhash + read dimensions
	const { thumbhash, width, height } = await computeImageMetadata(file);

	// 4. Confirm upload with metadata
	const confirmResult = await api.backgrounds.confirmUpload(id, {
		thumbhash,
		width,
		height,
		file_size_bytes: file.size,
		content_type: file.type
	});

	if (confirmResult.isErr) {
		throw new Error(confirmResult.error.message);
	}
}

async function computeImageMetadata(
	file: File
): Promise<{ thumbhash: string | undefined; width: number; height: number }> {
	return new Promise((resolve, reject) => {
		const img = new Image();
		img.onload = () => {
			const { naturalWidth: width, naturalHeight: height } = img;

			// Downscale for thumbhash (max 100px on longest side)
			const maxDim = 100;
			const scale = Math.min(maxDim / width, maxDim / height);
			const w = Math.round(width * scale);
			const h = Math.round(height * scale);

			const canvas = document.createElement('canvas');
			canvas.width = w;
			canvas.height = h;
			const ctx = canvas.getContext('2d');
			if (!ctx) {
				resolve({ thumbhash: undefined, width, height });
				return;
			}

			ctx.drawImage(img, 0, 0, w, h);
			const pixels = ctx.getImageData(0, 0, w, h);
			const hash = rgbaToThumbHash(w, h, pixels.data);
			const thumbhash = btoa(String.fromCharCode(...hash));

			resolve({ thumbhash, width, height });
			URL.revokeObjectURL(img.src);
		};
		img.onerror = () => {
			URL.revokeObjectURL(img.src);
			reject(new Error(`Failed to load image: ${file.name}`));
		};
		img.src = URL.createObjectURL(file);
	});
}

// -- Toggle / Update --

async function toggleEnabled(bg: Background) {
	actionLoading = true;
	const result = await api.backgrounds.update(bg.id, { enabled: !bg.enabled });
	if (result.isErr) {
		uploadError = result.error.message;
	}
	actionLoading = false;
	await refresh();
}

async function updateThemeMode(bg: Background, mode: ThemeMode) {
	actionLoading = true;
	const result = await api.backgrounds.update(bg.id, { theme_mode: mode });
	if (result.isErr) {
		uploadError = result.error.message;
	}
	actionLoading = false;
	await refresh();
}

// -- Delete --

function requestDelete(bg: Background) {
	deleteTarget = bg;
	showDeleteConfirm = true;
}

async function confirmDelete() {
	if (!deleteTarget) return;
	actionLoading = true;
	const result = await api.backgrounds.remove(deleteTarget.id);
	if (result.isErr) {
		uploadError = result.error.message;
	}
	deleteTarget = null;
	actionLoading = false;
	await refresh();
}

// -- Helpers --

function getThumbnailUrl(bg: Background): string | null {
	return cfImageUrl(bg.image_url, { width: 320, quality: 70, format: 'webp' });
}

function getThumbhashUrl(bg: Background): string | null {
	return decodeThumbhash(bg.thumbhash);
}

const THEME_MODE_OPTIONS: { value: ThemeMode; label: string }[] = [
	{ value: 'both', label: 'Both' },
	{ value: 'light', label: 'Light' },
	{ value: 'dark', label: 'Dark' }
];
</script>

<div class="space-y-6">
	<AdminPageHeader title="Settings" {refreshing} onrefresh={refresh} />

	<!-- Backgrounds Section -->
	<section class="space-y-4">
		<div class="flex items-center justify-between">
			<div>
				<h2 class="text-lg font-semibold">Backgrounds</h2>
				<p class="text-sm text-muted-foreground">
					Manage background images shown behind pages. {backgrounds.length} total, {backgrounds.filter(
						(b: Background) => b.enabled
					).length} enabled.
				</p>
			</div>
		</div>

		{#if error}
			<Alert variant="destructive">Error loading backgrounds: {error}</Alert>
		{/if}

		{#if uploadError}
			<Alert variant="destructive">{uploadError}</Alert>
		{/if}

		<!-- Upload Drop Zone -->
		<div
			class="relative rounded-lg border-2 border-dashed p-8 text-center transition-colors {dragOver
				? 'border-primary bg-primary/5'
				: 'border-border hover:border-muted-foreground/50'}"
			role="region"
			aria-label="Upload backgrounds"
			ondragover={handleDragOver}
			ondragleave={handleDragLeave}
			ondrop={handleDrop}
		>
			{#if uploading}
				<div class="flex flex-col items-center gap-2">
					<Upload class="h-8 w-8 animate-pulse text-muted-foreground" />
					<p class="text-sm text-muted-foreground">Uploading...</p>
				</div>
			{:else}
				<div class="flex flex-col items-center gap-2">
					<ImagePlus class="h-8 w-8 text-muted-foreground" />
					<p class="text-sm text-muted-foreground">
						Drag and drop images here, or
						<label class="cursor-pointer text-primary underline underline-offset-2">
							browse
							<input
								type="file"
								accept="image/*"
								multiple
								class="hidden"
								onchange={handleFileSelect}
							/>
						</label>
					</p>
					<p class="text-xs text-muted-foreground/70">Supports JPEG, PNG, WebP</p>
				</div>
			{/if}
		</div>

		<!-- Backgrounds Grid -->
		{#if backgrounds.length > 0}
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each backgrounds as bg (bg.id)}
					{@const thumbnailUrl = getThumbnailUrl(bg)}
					{@const thumbhashUrl = getThumbhashUrl(bg)}
					<div
						class="group relative overflow-hidden rounded-lg border bg-card transition-shadow hover:shadow-md {!bg.enabled
							? 'opacity-60'
							: ''}"
					>
						<!-- Image Preview -->
						<div class="relative aspect-video bg-muted">
							{#if thumbhashUrl}
								<img
									src={thumbhashUrl}
									alt=""
									class="absolute inset-0 h-full w-full object-cover"
									aria-hidden="true"
								/>
							{/if}
							{#if thumbnailUrl}
								<img
									src={thumbnailUrl}
									alt={bg.original_filename ?? 'Background'}
									class="relative h-full w-full object-cover"
									loading="lazy"
								/>
							{/if}

							<!-- Enabled/Disabled Badge -->
							<button
								class="absolute right-2 top-2 rounded-full p-1.5 transition-colors {bg.enabled
									? 'bg-green-500/80 text-white hover:bg-green-600'
									: 'bg-black/50 text-white opacity-70 hover:bg-black/70 hover:opacity-100'}"
								title={bg.enabled ? 'Enabled - click to disable' : 'Disabled - click to enable'}
								onclick={() => toggleEnabled(bg)}
								disabled={actionLoading}
							>
								{#if bg.enabled}
									<Eye class="h-4 w-4" />
								{:else}
									<EyeOff class="h-4 w-4" />
								{/if}
							</button>
						</div>

						<!-- Card Body -->
						<div class="space-y-2 p-3">
							<!-- Filename & Meta -->
							<div class="flex items-start justify-between gap-2">
								<div class="min-w-0 flex-1">
									<p class="truncate text-sm font-medium">
										{bg.original_filename ?? bg.id}
									</p>
									<p class="text-xs text-muted-foreground">
										{#if bg.width && bg.height}
											{bg.width}&times;{bg.height}
										{/if}
										{#if bg.file_size_bytes}
											&middot; {formatBytes(bg.file_size_bytes)}
										{/if}
									</p>
								</div>
							</div>

							<!-- Theme Mode Select -->
							<div class="flex items-center gap-2">
								<span class="text-xs text-muted-foreground">Theme:</span>
								<Select.Root
									type="single"
									value={bg.theme_mode}
								onValueChange={(v) => {
									if (v) void updateThemeMode(bg, v as ThemeMode);
								}}
								>
									<Select.Trigger class="h-7 w-24 text-xs">
										{THEME_MODE_OPTIONS.find((o) => o.value === bg.theme_mode)?.label ??
											bg.theme_mode}
									</Select.Trigger>
									<Select.Content>
										{#each THEME_MODE_OPTIONS as opt (opt.value)}
											<Select.Item value={opt.value}>{opt.label}</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>

								<!-- Delete Button -->
								<button
									class="ml-auto rounded p-1 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
									title="Delete background"
									onclick={() => requestDelete(bg)}
									disabled={actionLoading}
								>
									<Trash2 class="h-4 w-4" />
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{:else if !error}
			<div class="rounded-lg border border-border bg-card p-8 text-center">
				<ImagePlus class="mx-auto h-12 w-12 text-muted-foreground opacity-40" />
				<p class="mt-2 text-sm text-muted-foreground">
					No backgrounds yet. Upload some images to get started.
				</p>
			</div>
		{/if}
	</section>
</div>

<ConfirmDialog
	bind:open={showDeleteConfirm}
	title="Delete Background"
	description="Delete this background image? This cannot be undone."
	confirmLabel="Delete"
	onConfirm={confirmDelete}
/>
