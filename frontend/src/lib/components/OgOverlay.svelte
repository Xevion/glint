<script lang="ts">
interface Props {
	/** Primary title (shader name, scene name, or "Glint") */
	title: string;
	/** Secondary info shown to the right of the title (e.g. version) */
	subtitle?: string;
	/** Bottom-left metadata line (e.g. "Medieval Village · Ultra") */
	meta?: string;
}

let { title, subtitle, meta }: Props = $props();

const titleSize = $derived(title.length > 24 ? '44px' : title.length > 16 ? '52px' : '60px');
</script>

<!--
  Transparent overlay for OG images. Rendered via satori to SVG,
  then composited on top of the screenshot + gradient by sharp.
  
  Satori constraints: inline styles, flexbox only, no CSS variables.
-->
<div
	style="display: flex; flex-direction: column; justify-content: flex-end; width: 1200px; height: 630px; padding: 0 48px 40px 48px;"
>
	<!-- Title row -->
	<div
		style="display: flex; justify-content: space-between; align-items: baseline; width: 100%; margin-bottom: 12px;"
	>
		<div
			style="font-family: 'Geist Sans', sans-serif; font-weight: 900; font-size: {titleSize}; color: #ffffff; line-height: 1.15; max-width: 850px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
		>
			{title}
		</div>
		{#if subtitle}
			<div
				style="font-family: 'Inter', sans-serif; font-weight: 500; font-size: 28px; color: rgba(255, 255, 255, 0.7); white-space: nowrap; margin-left: 24px;"
			>
				{subtitle}
			</div>
		{/if}
	</div>

	<!-- Bottom metadata row -->
	<div
		style="display: flex; justify-content: space-between; align-items: center; width: 100%; border-top: 1px solid rgba(255, 255, 255, 0.15); padding-top: 16px;"
	>
		{#if meta}
			<div
				style="font-family: 'Inter', sans-serif; font-weight: 400; font-size: 24px; color: rgba(255, 255, 255, 0.55);"
			>
				{meta}
			</div>
		{:else}
			<div></div>
		{/if}
		<div
			style="font-family: 'Inter', sans-serif; font-weight: 500; font-size: 24px; color: rgba(255, 255, 255, 0.45);"
		>
			Glint
		</div>
	</div>
</div>
