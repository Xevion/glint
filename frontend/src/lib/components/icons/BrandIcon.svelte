<script lang="ts">
	import { icons, type IconName } from './index';

	interface Props {
		name: IconName;
		class?: string;
		/** Whether to show the brand color on hover */
		colorOnHover?: boolean;
	}

	let { name, class: className = 'h-4 w-4', colorOnHover = false }: Props = $props();

	const icon = $derived(icons[name]);
</script>

<svg class={className} viewBox={icon.viewBox} fill="currentColor">
	{#each icon.paths as path (path.d)}
		<path
			class={colorOnHover ? 'transition-colors' : undefined}
			style={colorOnHover ? `--hover-color: ${icon.hoverColor}` : undefined}
			fill-rule={path.fillRule ?? undefined}
			clip-rule={path.clipRule ?? undefined}
			d={path.d}
		/>
	{/each}
</svg>

<style>
	path.transition-colors {
		transition-property: fill;
		transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
		transition-duration: 150ms;
	}

	:global(.group\/modrinth:hover) path.transition-colors,
	:global(.group\/curseforge:hover) path.transition-colors {
		fill: var(--hover-color);
	}
</style>
