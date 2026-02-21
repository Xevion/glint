<script lang="ts">
import { resolve } from '$app/paths';
import CaptureImage from '$lib/components/CaptureImage.svelte';
import { ImageOverlay } from '$lib/components/ui/image-overlay';
import { cn } from '$lib/utils';

interface AuthorCardAuthor {
	name: string;
	slug: string;
	shaderCount: number;
	imagePath?: string | null;
	thumbhash?: string | null;
	topShaderName?: string | null;
}

interface Props {
	author: AuthorCardAuthor;
	class?: string;
}

let { author, class: className }: Props = $props();
</script>

<div
	class={cn(
		'group relative flex h-full flex-col overflow-hidden rounded-xl bg-card transition-all duration-300',
		'border border-border shadow-theme-sm',
		'hover:-translate-y-1 hover:border-primary/50 hover:shadow-theme-lg',
		className
	)}
>
	<!-- Thumbnail Image -->
	<div class="relative">
		<CaptureImage
			src={author.imagePath}
			thumbhash={author.thumbhash}
			preset="card"
			alt="{author.name} preview"
			class="h-full w-full object-cover"
			containerClass="aspect-video w-full overflow-hidden"
		/>
		<ImageOverlay />
	</div>

	<!-- Card Body -->
	<div class="flex flex-1 flex-col gap-1.5 px-3 pt-2.5 pb-2">
		<a
			href={resolve('/authors/[slug]', { slug: author.slug })}
			class="block truncate text-base font-semibold text-card-foreground transition-colors hover:text-primary focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-2 after:absolute after:inset-0 after:content-['']"
		>
			{author.name}
		</a>
		<span class="text-sm text-muted-foreground">
			{author.shaderCount} {author.shaderCount === 1 ? 'shader' : 'shaders'}
		</span>
		{#if author.topShaderName}
			<p class="truncate text-xs text-muted-foreground">
				Top shader: <span class="font-medium text-card-foreground">{author.topShaderName}</span>
			</p>
		{/if}
	</div>
</div>
