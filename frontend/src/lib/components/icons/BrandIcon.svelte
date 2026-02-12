<script lang="ts">
import { type IconName, icons } from './index';

interface Props {
	name: IconName;
	class?: string;
	/** Whether to show the brand color on hover */
	colorOnHover?: boolean;
}

let { name, class: className = 'h-4 w-4', colorOnHover = false }: Props = $props();

const icon = $derived(icons[name]);
</script>

<svg
	class="{className}{colorOnHover ? ' brand-icon-hover' : ''}"
	style={colorOnHover ? `--brand-color: ${icon.hoverColor}` : undefined}
	fill="currentColor"
	aria-hidden="true"
>
	<use href="#brand-{name}" />
</svg>

<style>
	.brand-icon-hover {
		transition-property: color;
		transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
		transition-duration: 150ms;
	}

	:global(.group\/modrinth:hover) .brand-icon-hover,
	:global(.group\/curseforge:hover) .brand-icon-hover {
		color: var(--brand-color);
	}
</style>
