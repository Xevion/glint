<script lang="ts">
import type { AdminShader } from './queries';

interface Props {
	shader: AdminShader;
}

let { shader }: Props = $props();
</script>

<span class="font-medium {shader.deletedAt ? 'opacity-50' : ''}">{shader.name}</span>
{#if shader.deletedAt}
	<span class="ml-1.5 shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive">Deleted</span>
{:else if !shader.captureEnabled}
	<span class="ml-1.5 shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-xs font-medium text-destructive">Disabled</span>
{/if}
