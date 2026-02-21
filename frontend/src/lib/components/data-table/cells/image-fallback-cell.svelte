<script lang="ts">
import type { Component } from 'svelte';

interface Props {
	icon: Component;
	sizeClass: string;
	roundedClass: string;
}

let { icon: Icon, sizeClass, roundedClass }: Props = $props();
</script>

<div class="flex {sizeClass} {roundedClass} shrink-0 items-center justify-center bg-muted pointer-events-none">
	<Icon class="h-4 w-4 text-muted-foreground" />
</div>
