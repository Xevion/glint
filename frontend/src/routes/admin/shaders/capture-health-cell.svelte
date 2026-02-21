<script lang="ts">
import type { AdminShader } from './queries';

interface Props {
	health?: AdminShader['captureHealth'];
}
let { health }: Props = $props();
</script>

{#if health == null || health.total === 0}
  <span class="text-muted-foreground">—</span>
{:else}
  {@const pct = health.completed / health.total}
  <span
    class="tabular-nums {pct >= 1
      ? 'text-green-600 dark:text-green-400'
      : pct >= 0.5
        ? 'text-amber-600 dark:text-amber-400'
        : 'text-red-600 dark:text-red-400'}"
  >
    {health.completed}/{health.total}
  </span>
{/if}
