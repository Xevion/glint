<script lang="ts">
import { fly, fade } from 'svelte/transition';
import { Button } from '$lib/components/ui/button';
import {
	ShaderCompare,
	ImagePicker,
	getMockCompareImages,
	type ImageOption,
	type ShaderDisplayInfo,
	type CompareMode
} from '$lib/components/compare';
import ErrorBoundaryFallback from '$lib/components/ErrorBoundaryFallback.svelte';
import { Columns3, SplitSquareHorizontal, ToggleLeft, ArrowLeftRight } from 'lucide-svelte';

// Get mock data from centralized location
const compareImages = getMockCompareImages();

// Transform CompareImage[] to ImageOption[] for ImagePicker
const testImages: (ImageOption & { shader: ShaderDisplayInfo })[] = compareImages.map((img) => ({
	url: img.url,
	label: img.shader.name,
	shader: img.shader
}));

let mode = $state<CompareMode>('slider');
let leftImage = $state(testImages[0].url);
let rightImage = $state(testImages[1].url);

const leftShader = $derived(testImages.find((img) => img.url === leftImage)?.shader);
const rightShader = $derived(testImages.find((img) => img.url === rightImage)?.shader);

function swapImages() {
	const temp = leftImage;
	leftImage = rightImage;
	rightImage = temp;
}

const modes: { value: CompareMode; label: string; icon: typeof Columns3 }[] = [
	{ value: 'slider', label: 'Slider', icon: Columns3 },
	{ value: 'split', label: 'Split', icon: SplitSquareHorizontal },
	{ value: 'toggle', label: 'Toggle', icon: ToggleLeft }
];

function setMode(newMode: CompareMode) {
	mode = newMode;
}
</script>

<div class="container mx-auto px-4 py-8">
	<div class="mx-auto max-w-4xl">
		<!-- Header -->
		<div in:fly={{ y: -10, duration: 400 }} class="mb-6">
			<h1 class="mb-2 text-3xl font-bold text-foreground">Compare Shaders</h1>
			<p class="text-muted-foreground">
				Compare shader screenshots with a slider, split view, or toggle between them.
			</p>
		</div>

		<!-- Mode selector -->
		<div in:fly={{ y: 10, duration: 400, delay: 50 }} class="mb-6">
			<div class="flex flex-wrap gap-2">
				{#each modes as { value, label, icon: Icon } (value)}
					<Button
						variant={mode === value ? 'default' : 'outline'}
						size="sm"
						onclick={() => setMode(value)}
						class="min-w-fit flex-1"
					>
						<Icon size={16} />
						{label}
					</Button>
				{/each}
			</div>
		</div>

		<!-- Shader comparison -->
		<div in:fade={{ duration: 300, delay: 100 }} class="mb-6 overflow-hidden rounded-xl border border-border bg-card">
			<svelte:boundary onerror={(e) => console.error('[Compare]', e)}>
				<ShaderCompare {leftImage} {rightImage} {mode} {leftShader} {rightShader} />

				{#snippet failed(error, reset)}
					<ErrorBoundaryFallback
						error={error instanceof Error ? error : new Error(String(error))}
						{reset}
						title="Comparison failed"
					/>
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
					images={testImages}
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
					images={testImages}
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
	</div>
</div>


