<script lang="ts">
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { MapPin } from '@lucide/svelte';
import type { AdminScene } from './+page';

interface Props {
	scene: AdminScene;
}

let { scene }: Props = $props();
</script>

{#if scene.imagePath ?? scene.thumbhash}
	<CaptureImage
		src={scene.imagePath}
		thumbhash={scene.thumbhash}
		preset="thumbnail"
		alt="{scene.name} preview"
		class="h-full w-full object-cover"
		containerClass="h-12 w-20 rounded pointer-events-none"
	/>
{:else}
	<div
		class="flex h-12 w-20 items-center justify-center rounded bg-muted"
	>
		<MapPin class="h-4 w-4 text-muted-foreground opacity-40" />
	</div>
{/if}
