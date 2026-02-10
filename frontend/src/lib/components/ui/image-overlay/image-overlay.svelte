<script lang="ts">
import { cn } from '$lib/utils';

interface Props {
	class?: string;
	hover?: boolean;
}
let { class: className, hover = false }: Props = $props();
</script>

<div
	class={cn(
		'absolute inset-0 bg-linear-to-t from-black/60 to-transparent',
		!hover && 'via-transparent',
		hover && 'opacity-0 transition-opacity group-hover:opacity-100',
		className
	)}
></div>
