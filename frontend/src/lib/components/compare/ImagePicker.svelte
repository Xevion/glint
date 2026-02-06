<script lang="ts">
import { cfImageUrl } from '$lib/utils/image';
import { ChevronDown } from '@lucide/svelte';
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

<div class="image-picker">
	{#if label}
		<span class="picker-label">{label}</span>
	{/if}

	<Popover.Root bind:open>
		<Popover.Trigger class="picker-trigger">
			<div class="trigger-content">
				<div class="thumbnail-wrapper">
					<img src={cfImageUrl(value, 'thumbnail') ?? value} alt={currentImage?.label ?? 'Selected'} class="thumbnail" />
				</div>
				<span class="current-label">{currentImage?.label ?? 'Select image'}</span>
				<ChevronDown class="chevron" size={16} />
			</div>
		</Popover.Trigger>

		<Popover.Content class="picker-content" sideOffset={8} align="start">
			<div class="image-grid">
				{#each images as image (image.url)}
					<button
						type="button"
						class="grid-item"
						class:selected={image.url === value}
						onclick={() => selectImage(image.url)}
					>
						<img src={cfImageUrl(image.url, 'thumbnail') ?? image.url} alt={image.label} />
						<span class="grid-label">{image.label}</span>
					</button>
				{/each}
			</div>
		</Popover.Content>
	</Popover.Root>
</div>

<style>
	.image-picker {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.picker-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: hsl(var(--foreground));
	}

	:global(.picker-trigger) {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem;
		background: hsl(var(--card));
		border: 1px solid hsl(var(--border));
		border-radius: var(--radius);
		cursor: pointer;
		transition: all 150ms ease;
		width: 100%;
	}

	:global(.picker-trigger:hover) {
		background: hsl(var(--accent));
		border-color: hsl(var(--accent));
	}

	:global(.picker-trigger:focus-visible) {
		outline: 2px solid hsl(var(--ring));
		outline-offset: 2px;
	}

	.trigger-content {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		width: 100%;
	}

	.thumbnail-wrapper {
		width: 3rem;
		height: 2rem;
		border-radius: calc(var(--radius) - 2px);
		overflow: hidden;
		flex-shrink: 0;
	}

	.thumbnail {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.current-label {
		flex: 1;
		text-align: left;
		font-size: 0.875rem;
		color: hsl(var(--foreground));
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.chevron) {
		color: hsl(var(--muted-foreground));
		flex-shrink: 0;
	}

	:global(.picker-content) {
		background: hsl(var(--popover));
		border: 1px solid hsl(var(--border));
		border-radius: var(--radius);
		padding: 0.5rem;
		box-shadow:
			0 4px 6px -1px rgb(0 0 0 / 0.1),
			0 2px 4px -2px rgb(0 0 0 / 0.1);
		z-index: 50;
		max-height: 300px;
		overflow-y: auto;
	}

	.image-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 0.5rem;
		min-width: 240px;
	}

	.grid-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.25rem;
		background: transparent;
		border: 2px solid transparent;
		border-radius: calc(var(--radius) - 2px);
		cursor: pointer;
		transition: all 150ms ease;
	}

	.grid-item:hover {
		background: hsl(var(--accent));
	}

	.grid-item:focus-visible {
		outline: 2px solid hsl(var(--ring));
		outline-offset: 1px;
	}

	.grid-item.selected {
		border-color: hsl(var(--primary));
		background: hsl(var(--primary) / 0.1);
	}

	.grid-item img {
		width: 100%;
		height: 48px;
		object-fit: cover;
		border-radius: calc(var(--radius) - 4px);
	}

	.grid-label {
		font-size: 0.625rem;
		color: hsl(var(--muted-foreground));
		text-align: center;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
