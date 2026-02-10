<script lang="ts">
interface Props {
	sceneName?: string | null;
	shaderName?: string | null;
	profile?: string | null;
	version?: string | null;
	formatVersion?: (v: string) => string;
	size?: 'sm' | 'xs';
}
let { sceneName, shaderName, profile, version, formatVersion: fmt, size = 'xs' }: Props = $props();

const py = $derived(size === 'sm' ? 'py-1 text-sm' : 'py-0.5 text-xs');
</script>

{#if shaderName}
	<div class="mb-1 font-bold text-white">{shaderName}</div>
{/if}
<div class="flex items-center gap-2">
	{#if sceneName}
		<span class="rounded bg-white/20 px-2 {py} font-medium text-white">
			{sceneName}
		</span>
	{/if}
	{#if profile}
		<span class="rounded bg-primary px-2 {py} font-medium text-white">
			{profile}
		</span>
	{/if}
	{#if version}
		<span class="rounded bg-white/20 px-2 {py} font-medium text-white">
			{fmt ? fmt(version) : `v${version}`}
		</span>
	{/if}
</div>
