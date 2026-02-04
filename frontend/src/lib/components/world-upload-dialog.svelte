<script lang="ts">
import { Button } from '$lib/components/ui/button';
import * as Dialog from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { Textarea } from '$lib/components/ui/textarea';
import Progress from '$lib/components/ui/progress/progress.svelte';
import { Plus, Folder, FileArchive } from 'lucide-svelte';
import { api } from '$lib/api';
import {
	compressDirectory,
	calculateDirectorySize,
	validateFileSize,
	type CompressionProgress
} from '$lib/utils/compression';
import { calculateSHA256 } from '$lib/utils/hash';

interface Props {
	onWorldCreated?: () => void;
}

let { onWorldCreated }: Props = $props();

type Step =
	| 'select'
	| 'compress'
	| 'metadata'
	| 'hashing'
	| 'preparing'
	| 'upload'
	| 'finalizing'
	| 'error';

let open = $state(false);
let step = $state<Step>('select');
let selectedFile = $state<File | null>(null);
let selectedDirectory = $state<FileSystemDirectoryHandle | null>(null);
let compressionProgress = $state(0);
let hashingProgress = $state(0);
let uploadProgress = $state(0);
let compressedBlob = $state<Blob | null>(null);
let error = $state<string | null>(null);

// Form fields
let name = $state('');
let slug = $state('');
let description = $state('');
let minecraftVersion = $state('1.21.4');
let slugManuallyEdited = $state(false);

// Capabilities detection
const supportsFileSystemAccess = 'showDirectoryPicker' in window;

// Computed weighted progress for upload phase
// Hashing: 10%, Preparing: 5%, Upload: 80%, Finalizing: 5%
let overallProgress = $derived.by(() => {
	if (step === 'compress') return compressionProgress * 0.1;
	if (step === 'hashing') return 0.1 + hashingProgress * 0.1;
	if (step === 'preparing') return 0.2;
	if (step === 'upload') return 0.2 + uploadProgress * 0.75;
	if (step === 'finalizing') return 0.95;
	return 0;
});

// Auto-generate slug from name (kebab-case)
function generateSlug(input: string): string {
	return input
		.toLowerCase()
		.trim()
		.replace(/[^a-z0-9\s-]/g, '')
		.replace(/\s+/g, '-')
		.replace(/-+/g, '-')
		.replace(/^-|-$/g, '');
}

// Auto-calculated slug based on name (shown when slug hasn't been manually edited)
let autoSlug = $derived(generateSlug(name));

function handleNameChange() {
	if (!slugManuallyEdited) {
		slug = autoSlug;
	}
}

function handleSlugChange() {
	slugManuallyEdited = true;
}

function validateZipFile(file: File): boolean {
	if (!file.name.endsWith('.zip')) {
		error = 'Please select a .zip file';
		return false;
	}
	if (!validateFileSize(file.size)) {
		error = 'File exceeds 512 MiB limit';
		return false;
	}
	return true;
}

function handleFileSelect(event: Event) {
	const input = event.target as HTMLInputElement;
	const file = input.files?.[0];
	if (!file) return;

	if (!validateZipFile(file)) {
		step = 'error';
		return;
	}

	selectedFile = file;
	step = 'metadata';
}

async function handleDirectorySelect() {
	try {
		const dirHandle = await window.showDirectoryPicker();
		const size = await calculateDirectorySize(dirHandle);

		if (!validateFileSize(size)) {
			error = 'Directory contents exceed 512 MiB limit';
			step = 'error';
			return;
		}

		selectedDirectory = dirHandle;
		step = 'compress';
		await startCompression();
	} catch (e) {
		if (e instanceof Error && e.name !== 'AbortError') {
			error = `Failed to select directory: ${e.message}`;
			step = 'error';
		}
	}
}

async function startCompression() {
	if (!selectedDirectory) return;

	try {
		const result = await compressDirectory(selectedDirectory, (progress: CompressionProgress) => {
			compressionProgress = progress.progress;
		});

		compressedBlob = result.blob;
		step = 'metadata';
	} catch (e) {
		error = e instanceof Error ? e.message : 'Compression failed';
		step = 'error';
	}
}

async function handleSubmit() {
	if (!name || !minecraftVersion) {
		error = 'Please fill in all required fields';
		return;
	}

	const fileToUpload = selectedFile ?? compressedBlob;
	if (!fileToUpload) {
		error = 'No file to upload';
		return;
	}

	// Auto-generate slug if not manually provided
	const finalSlug = slug.trim() || autoSlug;
	if (!finalSlug) {
		error = 'Could not generate slug from name';
		return;
	}

	try {
		// Step 1: Compute hash
		step = 'hashing';
		hashingProgress = 0;
		const hash = await calculateSHA256(fileToUpload);
		const hashWithPrefix = `sha256:${hash}`;
		hashingProgress = 1;

		// Step 2: Request presigned URL
		step = 'preparing';
		const prepareResult = await api.worlds.createWorldUpload({
			name,
			slug: finalSlug,
			description: description || undefined,
			minecraft_version: minecraftVersion,
			file_hash: hashWithPrefix,
			file_size_bytes: fileToUpload.size
		});

		if (prepareResult.isErr) {
			error = prepareResult.error.message;
			step = 'error';
			return;
		}

		const { upload_id, presigned_url } = prepareResult.value;

		// Step 3: Upload to R2
		step = 'upload';
		uploadProgress = 0;
		const uploadResult = await api.worlds.uploadToPresignedUrl(
			presigned_url,
			fileToUpload,
			hash,
			(progress) => {
				uploadProgress = progress.percentage / 100;
			}
		);

		if (uploadResult.isErr) {
			error = uploadResult.error.message;
			step = 'error';
			return;
		}

		// Step 4: Complete upload
		step = 'finalizing';
		const completeResult = await api.worlds.completeWorldUpload(finalSlug, { upload_id });

		if (completeResult.isErr) {
			error = completeResult.error.message;
			step = 'error';
			return;
		}

		// Success!
		resetAndClose();
		onWorldCreated?.();
	} catch (e) {
		error = e instanceof Error ? e.message : 'Upload failed';
		step = 'error';
	}
}

function resetAndClose() {
	open = false;
	setTimeout(() => {
		step = 'select';
		selectedFile = null;
		selectedDirectory = null;
		compressionProgress = 0;
		hashingProgress = 0;
		uploadProgress = 0;
		compressedBlob = null;
		error = null;
		name = '';
		slug = '';
		description = '';
		minecraftVersion = '1.21.4';
		slugManuallyEdited = false;
	}, 200);
}

function handleBack() {
	if (step === 'metadata') {
		step = 'select';
		selectedFile = null;
		selectedDirectory = null;
		compressedBlob = null;
	} else if (step === 'error') {
		step = 'metadata';
		error = null;
	}
}

function getStepDescription(currentStep: Step): string {
	switch (currentStep) {
		case 'select':
			return 'Select a world folder or .zip file to upload';
		case 'compress':
			return 'Compressing world files...';
		case 'metadata':
			return 'Fill in world details';
		case 'hashing':
			return 'Computing file hash...';
		case 'preparing':
			return 'Preparing upload...';
		case 'upload':
			return 'Uploading world to storage...';
		case 'finalizing':
			return 'Finalizing world creation...';
		case 'error':
			return 'Upload failed';
	}
}
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger>
		<Button><Plus class="mr-2 h-4 w-4" />Create World</Button>
	</Dialog.Trigger>

	<Dialog.Content class="sm:max-w-125">
		<Dialog.Header>
			<Dialog.Title>Create World</Dialog.Title>
			<Dialog.Description>
				{getStepDescription(step)}
			</Dialog.Description>
		</Dialog.Header>

		<div class="grid gap-4 py-4">
			{#if step === 'select'}
				<div class="grid gap-4">
					{#if supportsFileSystemAccess}
						<Button
							variant="outline"
							onclick={handleDirectorySelect}
							class="h-24 w-full flex-col gap-2"
						>
							<Folder class="h-8 w-8" />
							<span>Select Folder</span>
						</Button>

						<div class="relative">
							<div class="absolute inset-0 flex items-center">
								<span class="w-full border-t"></span>
							</div>
							<div class="relative flex justify-center text-xs uppercase">
								<span class="bg-background px-2 text-muted-foreground">Or</span>
							</div>
						</div>
					{/if}

					<Button
						variant="outline"
						onclick={() => document.getElementById('world-zip-upload')?.click()}
						class="h-24 w-full flex-col gap-2"
					>
						<FileArchive class="h-8 w-8" />
						<span>Select ZIP File</span>
					</Button>
					<input
						id="world-zip-upload"
						type="file"
						accept=".zip"
						onchange={handleFileSelect}
						class="hidden"
						aria-label="Upload ZIP file"
					/>
				</div>
			{:else if step === 'compress'}
				<div class="space-y-4">
					<Progress value={compressionProgress * 100} class="h-2" />
					<p class="text-center text-sm text-muted-foreground">
						Compressing files... {Math.round(compressionProgress * 100)}%
					</p>
				</div>
			{:else if step === 'metadata'}
				<div class="grid gap-4">
					<div class="grid gap-2">
						<Label for="name">Name *</Label>
						<Input
							id="name"
							bind:value={name}
							oninput={handleNameChange}
							placeholder="My Minecraft World"
							required
						/>
					</div>

					<div class="grid gap-2">
						<Label for="slug">Slug</Label>
						<Input
							id="slug"
							bind:value={slug}
							oninput={handleSlugChange}
							placeholder="my-minecraft-world"
						/>
					</div>

					<div class="grid gap-2">
						<Label for="minecraft_version">Minecraft Version *</Label>
						<Input
							id="minecraft_version"
							bind:value={minecraftVersion}
							placeholder="1.21.4"
							required
						/>
					</div>

					<div class="grid gap-2">
						<Label for="description">Description</Label>
						<Textarea
							id="description"
							bind:value={description}
							placeholder="A beautiful world with custom terrain..."
							rows={3}
						/>
					</div>
				</div>
			{:else if step === 'hashing' || step === 'preparing' || step === 'upload' || step === 'finalizing'}
				<div class="space-y-4">
					<Progress value={overallProgress * 100} class="h-2" />
					<p class="text-center text-sm text-muted-foreground">
						{#if step === 'hashing'}
							Computing file hash...
						{:else if step === 'preparing'}
							Preparing upload...
						{:else if step === 'upload'}
							Uploading... {Math.round(uploadProgress * 100)}%
						{:else if step === 'finalizing'}
							Finalizing...
						{/if}
					</p>
				</div>
			{:else if step === 'error'}
				<div
					class="space-y-2 rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive"
				>
					<p class="font-semibold">Upload Failed</p>
					<p class="text-sm">{error ?? 'An unknown error occurred'}</p>
					<p class="text-xs text-destructive/80">
						Please check your input and try again, or contact support if the issue persists.
					</p>
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			{#if step === 'select' || step === 'metadata' || step === 'error'}
				<Button variant="outline" onclick={step === 'error' ? handleBack : resetAndClose}>
					{step === 'error' ? 'Back' : 'Cancel'}
				</Button>
			{/if}
			{#if step === 'metadata'}
				<Button onclick={handleSubmit} disabled={!name || !minecraftVersion}>Create World</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
