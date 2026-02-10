<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ChevronDown, Check } from '@lucide/svelte';
import { Popover } from 'bits-ui';
import type { ImageOption } from './types';

interface Props {
	value: string;
	images: ImageOption[];
	onSelect: (url: string) => void;
	label?: string;
}

let { value, images, onSelect, label }: Props = $props();

let open = $state(false);

const currentImage = $derived(images.find((img) => img.url === value));

function selectImage(url: string) {
	onSelect(url);
	open = false;
}
</script>

<div class="flex flex-col gap-1.5">
	{#if label}
		<span class="text-sm font-medium text-foreground">{label}</span>
	{/if}

	<Popover.Root bind:open>
		<Popover.Trigger
			class="inline-flex w-full items-center gap-3 rounded-md border border-input bg-background px-3 py-2 shadow-theme-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring dark:border-input dark:bg-input/30 dark:hover:bg-input/50"
		>
			<CaptureImage
				src={value}
				thumbhash={currentImage?.thumbhash}
				preset="thumbnail"
				alt={currentImage?.label ?? 'Selected'}
				class="h-full w-full object-cover"
				containerClass="h-8 w-12 flex-shrink-0 rounded-sm"
			/>
			<span class="flex-1 truncate text-left text-sm text-foreground">
				{currentImage?.label ?? 'Select image'}
			</span>
			<ChevronDown class="size-4 flex-shrink-0 text-muted-foreground transition-transform {open ? 'rotate-180' : ''}" />
		</Popover.Trigger>

		<Popover.Content
			class="z-50 max-h-[400px] w-[360px] overflow-y-auto rounded-md border border-border/60 bg-popover p-2 shadow-theme-sm"
			sideOffset={8}
			align="start"
		>
			<div class="grid grid-cols-2 gap-2">
				{#each images as image (image.url)}
					{@const selected = image.url === value}
					<button
						type="button"
						class="group flex flex-col gap-1.5 rounded-md p-1.5 transition-colors
							{selected
								? 'bg-primary/10 ring-1 ring-primary/50'
								: 'hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'}"
						onclick={() => selectImage(image.url)}
					>
						<div class="relative">
							<CaptureImage
								src={image.url}
								thumbhash={image.thumbhash}
								preset="card"
								alt={image.label}
								class="h-full w-full object-cover transition-transform group-hover:scale-105"
								containerClass="aspect-video rounded-sm"
							/>
							{#if selected}
								<div class="absolute right-1 top-1 flex size-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-theme-sm">
									<Check class="size-3" />
								</div>
							{/if}
						</div>
						<span
							class="truncate text-center text-xs {selected ? 'font-medium text-foreground' : 'text-muted-foreground'}"
						>
							{image.label}
						</span>
					</button>
				{/each}
			</div>
		</Popover.Content>
	</Popover.Root>
</div>
