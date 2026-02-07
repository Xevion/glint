<script lang="ts">
import { api } from '$lib/api';
import type { AdoptPreviewResponse, Shader } from '$lib/bindings';
import { Button } from '$lib/components/ui/button';
import * as Dialog from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import { ArrowLeft, Check, CircleAlert, Download, Info, LoaderCircle } from '@lucide/svelte';
import type { Snippet } from 'svelte';

const PLATFORM_URL_PATTERN =
	/^https?:\/\/(www\.)?(modrinth\.com\/shader\/|curseforge\.com\/minecraft\/shaders\/)/i;

interface Props {
	onShaderAdopted?: (shader: Shader) => void;
	/** If provided, skip URL input and go straight to preview when opened */
	initialUrl?: string;
	/** Optional trigger snippet; if omitted, renders default "Adopt Shader" button */
	trigger?: Snippet;
	/** Bindable open state for external control */
	open?: boolean;
}

let { onShaderAdopted, initialUrl, trigger, open = $bindable(false) }: Props = $props();

let step = $state<'url' | 'preview'>('url');
let loading = $state(false);
let adopting = $state(false);
let error = $state<string | null>(null);
let errorIsConflict = $state(false);
let url = $state('');
let preview = $state<AdoptPreviewResponse | null>(null);
let initialized = $state(false);

$effect(() => {
	if (open && initialUrl && !initialized) {
		initialized = true;
		url = initialUrl;
		void handlePreview();
	}
	if (!open) {
		initialized = false;
	}
});

function isValidPlatformUrl(input: string): boolean {
	return PLATFORM_URL_PATTERN.test(input.trim());
}

async function handlePreview() {
	const trimmed = url.trim();
	if (!trimmed) return;

	if (!isValidPlatformUrl(trimmed)) {
		error = 'Please enter a valid Modrinth or CurseForge shader URL.';
		errorIsConflict = false;
		return;
	}

	loading = true;
	error = null;
	errorIsConflict = false;

	const result = await api.adopt.preview(trimmed);
	if (result.isOk) {
		preview = result.value;
		step = 'preview';
	} else {
		error = result.error.message;
		errorIsConflict = false;
	}
	loading = false;
}

async function handleAdopt() {
	if (!url.trim()) return;

	adopting = true;
	error = null;
	errorIsConflict = false;

	const result = await api.adopt.adopt(url.trim());
	if (result.isOk) {
		const shader = result.value;
		resetAndClose();
		onShaderAdopted?.(shader);
	} else {
		const isAlreadyAdopted = result.error.statusCode === 409 || result.error.code === 'CONFLICT';
		if (isAlreadyAdopted) {
			error = 'This shader is already in Glint.';
			errorIsConflict = true;
		} else {
			error = result.error.message;
			errorIsConflict = false;
		}
		adopting = false;
	}
}

function goBack() {
	step = 'url';
	preview = null;
	error = null;
	errorIsConflict = false;
}

function resetAndClose() {
	open = false;
	step = 'url';
	url = '';
	preview = null;
	error = null;
	errorIsConflict = false;
	loading = false;
	adopting = false;
}

function handleOpenChange(newOpen: boolean) {
	if (!newOpen) {
		resetAndClose();
	} else {
		open = true;
	}
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter' && step === 'url' && !loading) {
		void handlePreview();
	}
}

function formatNumber(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toLocaleString();
}
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
	{#if trigger}
		<Dialog.Trigger>
			{@render trigger()}
		</Dialog.Trigger>
	{:else}
		<Dialog.Trigger>
			{#snippet child({ props })}
				<Button variant="outline" {...props}>
					<Download class="mr-2 h-4 w-4" />
					Adopt Shader
				</Button>
			{/snippet}
		</Dialog.Trigger>
	{/if}
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>
				{#if step === 'url'}
					Adopt Shader
				{:else}
					Confirm Adoption
				{/if}
			</Dialog.Title>
			<Dialog.Description>
				{#if step === 'url'}
					Paste a Modrinth or CurseForge shader URL to adopt it into Glint.
				{:else}
					Review shader details before adopting.
				{/if}
			</Dialog.Description>
		</Dialog.Header>

		{#if error}
			{#if errorIsConflict}
				<div
					class="flex items-start gap-2 rounded-lg border border-blue-200 bg-blue-50 p-3 text-sm text-blue-800 dark:border-blue-800/40 dark:bg-blue-950/30 dark:text-blue-300"
				>
					<Info class="mt-0.5 h-4 w-4 shrink-0" />
					<div>
						{error}
						<a href="/admin/shaders" class="ml-1 underline hover:no-underline">
							View adopted shaders
						</a>
					</div>
				</div>
			{:else}
				<div
					class="flex items-start gap-2 rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
				>
					<CircleAlert class="mt-0.5 h-4 w-4 shrink-0" />
					<div>{error}</div>
				</div>
			{/if}
		{/if}

		{#if step === 'url'}
			<div class="grid gap-4 py-4">
				<div class="grid gap-2">
					<Label for="shader-url">Platform URL</Label>
					<Input
						id="shader-url"
						placeholder="https://modrinth.com/shader/bsl-shaders"
						bind:value={url}
						onkeydown={handleKeydown}
						disabled={loading}
					/>
					<p class="text-xs text-muted-foreground">
						Supports Modrinth and CurseForge shader URLs
					</p>
				</div>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
				<Button onclick={handlePreview} disabled={loading || !url.trim()}>
					{#if loading}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Preview
				</Button>
			</Dialog.Footer>
		{:else if preview}
			<div class="py-4 space-y-4">
				<div class="flex items-start gap-3">
					{#if preview.icon_url}
						<img
							src={preview.icon_url}
							alt={preview.name}
							class="h-12 w-12 rounded-lg object-cover"
						/>
					{:else}
						<div class="h-12 w-12 rounded-lg bg-muted flex items-center justify-center">
							<Download class="h-6 w-6 text-muted-foreground" />
						</div>
					{/if}
					<div class="min-w-0 flex-1">
						<h3 class="font-medium">{preview.name}</h3>
						<p class="text-sm text-muted-foreground line-clamp-2">{preview.description}</p>
					</div>
				</div>

				<dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
					<dt class="text-muted-foreground">Platform</dt>
					<dd class="capitalize">{preview.platform}</dd>

					<dt class="text-muted-foreground">Slug</dt>
					<dd class="font-mono text-xs">{preview.slug}</dd>

					<dt class="text-muted-foreground">Downloads</dt>
					<dd>{formatNumber(preview.downloads)}</dd>

					<dt class="text-muted-foreground">Versions</dt>
					<dd>{preview.version_count}</dd>

					{#if preview.authors.length > 0}
						<dt class="text-muted-foreground">Authors</dt>
						<dd>{preview.authors.map((a) => a.name).join(', ')}</dd>
					{/if}
				</dl>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={goBack} disabled={adopting}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back
				</Button>
				<Button onclick={handleAdopt} disabled={adopting}>
					{#if adopting}
						<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
					{:else}
						<Check class="mr-2 h-4 w-4" />
					{/if}
					Adopt
				</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
