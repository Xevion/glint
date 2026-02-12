<script lang="ts">
import { goto } from '$app/navigation';
import ErrorCard from '$lib/components/ErrorCard.svelte';
import Meta from '$lib/components/Meta.svelte';
import {
	type CompareMode,
	type ImageOption,
	ImagePicker,
	ShaderCompare,
	type ShaderDisplayInfo
} from '$lib/components/compare';
import { Alert } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { ArrowLeftRight, Columns3, SplitSquareHorizontal, ToggleLeft } from '@lucide/svelte';
import { fade, fly } from 'svelte/transition';
import type { PageData } from './$types';

interface Props {
	data: PageData;
}
let { data }: Props = $props();

type CompareImageOption = ImageOption & { shader: ShaderDisplayInfo };

const images: CompareImageOption[] = $derived(
	data.captures.map((c) => {
		const suffix = [c.shader_version, c.profile_name].filter(Boolean).join(' · ');
		return {
			url: c.image_url,
			label: suffix ? `${c.shader_name} (${suffix})` : c.shader_name,
			thumbhash: c.thumbhash,
			shader: {
				name: c.shader_name,
				version: c.shader_version,
				author: c.shader_author ?? null,
				profile_name: c.profile_name
			}
		};
	})
);

let mode = $state<CompareMode>('slider');
let leftImage = $state<string | undefined>(undefined);
let rightImage = $state<string | undefined>(undefined);

// Set defaults only on initial load — don't clobber user selections on re-derive
$effect(() => {
	if (leftImage !== undefined && rightImage !== undefined) return;
	if (images.length >= 2) {
		leftImage ??= images[0].url;
		rightImage ??= images[1].url;
	} else if (images.length === 1) {
		leftImage ??= images[0].url;
		rightImage ??= images[0].url;
	}
});

const leftShader = $derived(images.find((img) => img.url === leftImage)?.shader);
const rightShader = $derived(images.find((img) => img.url === rightImage)?.shader);

function swapImages() {
	const temp = leftImage;
	leftImage = rightImage;
	rightImage = temp;
}

function selectScene(slug: string) {
	void goto(`?scene=${encodeURIComponent(slug)}`, { replaceState: true });
}

const modes: { value: CompareMode; label: string; icon: typeof Columns3 }[] = [
	{ value: 'slider', label: 'Slider', icon: Columns3 },
	{ value: 'split', label: 'Split', icon: SplitSquareHorizontal },
	{ value: 'toggle', label: 'Toggle', icon: ToggleLeft }
];

// OG metadata: build a dynamic title and pick the first capture as the image
const ogTitle = $derived.by(() => {
	if (images.length >= 2) {
		const left = images[0].shader.name;
		const right = images[1].shader.name;
		const scene = data.scenes.find((s) => s.slug === data.selectedSceneSlug);
		const scenePart = scene ? ` on ${scene.name}` : '';
		return `${left} vs ${right}${scenePart}`;
	}
	return 'Compare Shaders';
});
const ogImage = $derived(data.captures[0]?.image_url ?? null);
const ogImagePath = $derived(
	data.selectedSceneSlug ? `/og/compare/${data.selectedSceneSlug}/og.png` : null
);
const ogDescription = $derived.by(() => {
	if (images.length >= 2) {
		return `Side-by-side comparison of ${images[0].shader.name} and ${images[1].shader.name} shaders for Minecraft.`;
	}
	return 'Side-by-side shader comparison for Minecraft. Compare screenshots with a slider, split view, or toggle.';
});
</script>

<Meta title={ogTitle} description={ogDescription} image={ogImage} ogImagePath={ogImagePath} />

<div class="py-8">
	<div class="mx-auto max-w-4xl">
		<!-- Header -->
		<div in:fly={{ y: -10, duration: 400 }} class="mb-6">
			<h1 class="mb-2 text-3xl font-bold text-foreground">Compare Shaders</h1>
		<p class="text-muted-foreground">
			Compare shader screenshots with a slider, split view, or toggle between them.
		</p>
		</div>

		<!-- Scene selector + mode selector -->
		<div in:fly={{ y: 10, duration: 400, delay: 50 }} class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
			{#if data.scenes.length > 0}
			<select
				class="h-9 rounded-md border border-input bg-background px-3 text-sm shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
				value={data.selectedSceneSlug}
				onchange={(e) => selectScene(e.currentTarget.value)}
			>
					{#each data.scenes as scene (scene.slug)}
						<option value={scene.slug}>
							{scene.name} ({scene.capture_count})
						</option>
					{/each}
				</select>
			{/if}

			<div class="flex flex-wrap gap-2">
				{#each modes as { value, label, icon: Icon } (value)}
					<Button
						variant={mode === value ? 'default' : 'outline'}
						size="sm"
						onclick={() => (mode = value)}
						class="min-w-fit flex-1"
					>
						<Icon size={16} />
						{label}
					</Button>
				{/each}
			</div>
		</div>

		{#if data.error}
			<Alert variant="destructive" class="mb-6">
				Failed to load captures: {data.error}
			</Alert>
		{/if}

		{#if images.length >= 2 && leftImage && rightImage}
			<!-- Shader comparison -->
			<div in:fade={{ duration: 300, delay: 100 }} class="mb-6 overflow-hidden rounded-xl border border-border bg-card">
				<svelte:boundary onerror={(e) => console.error('[Compare]', e)}>
					<ShaderCompare {leftImage} {rightImage} {mode} {leftShader} {rightShader} />

				{#snippet failed(error, reset)}
					{@const err = error instanceof Error ? error : new Error(String(error))}
					<ErrorCard
						title="Comparison failed"
						message={err.message}
						stack={err.stack}
					>
						{#snippet actions()}
							<Button variant="outline" size="sm" onclick={reset}>Reload</Button>
						{/snippet}
					</ErrorCard>
				{/snippet}
				</svelte:boundary>
			</div>

			<!-- Image pickers -->
			<div
				in:fly={{ y: 10, duration: 400, delay: 150 }}
				class="flex flex-col items-stretch gap-4 sm:flex-row sm:items-end"
			>
				<div class="flex-1">
					<ImagePicker
						value={leftImage}
						{images}
						onSelect={(url: string) => (leftImage = url)}
						label="Left Shader"
					/>
				</div>

				<Button
					variant="ghost"
					size="icon"
					onclick={swapImages}
					class="mb-1 min-h-11 min-w-11 self-center"
				>
					<ArrowLeftRight size={20} />
				</Button>

				<div class="flex-1">
					<ImagePicker
						value={rightImage}
						{images}
						onSelect={(url: string) => (rightImage = url)}
						label="Right Shader"
					/>
				</div>
			</div>

			<!-- Help text -->
			<div in:fade={{ duration: 300, delay: 200 }} class="mt-6 text-center text-sm text-muted-foreground">
				{#if mode === 'slider'}
					Drag the slider to reveal one shader over the other
				{:else if mode === 'split'}
					Drag to pan, scroll to zoom. Both sides stay synchronized.
				{:else if mode === 'toggle'}
					Click the image to toggle between shaders
				{/if}
			</div>
		{:else if !data.error}
			<div class="flex flex-col items-center justify-center rounded-xl border border-border bg-card py-16 text-center">
				<p class="text-lg text-muted-foreground">No captures available for this scene.</p>
				<p class="mt-1 text-sm text-muted-foreground">
					At least two captures are needed to compare shaders.
				</p>
			</div>
		{/if}
	</div>
</div>
