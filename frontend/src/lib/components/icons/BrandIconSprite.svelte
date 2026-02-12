<!--
  SVG sprite containing brand icon symbols. Rendered once in the root layout
  so that BrandIcon components can reference them via <use href="#brand-{name}">.
-->
<script lang="ts">
import { icons } from './index';
</script>

<svg style="display:none" aria-hidden="true">
	{#each Object.entries(icons) as [name, icon] (name)}
		<symbol id="brand-{name}" viewBox={icon.viewBox}>
			{#each icon.paths as path (path.d)}
				<path
					fill-rule={path.fillRule ?? undefined}
					clip-rule={path.clipRule ?? undefined}
					d={path.d}
				/>
			{/each}
		</symbol>
	{/each}
</svg>
