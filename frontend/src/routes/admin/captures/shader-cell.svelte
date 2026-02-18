<script lang="ts">
import type { CaptureWithContext } from '$lib/bindings';

interface Props {
	capture: CaptureWithContext;
}

let { capture }: Props = $props();
</script>

<div>
	<a
		href="/shaders/{capture.shader_slug}"
		class="font-medium text-primary hover:underline"
		onclick={(e) => e.stopPropagation()}>{capture.shader_name}</a
	>
	<div class="text-xs text-muted-foreground">{capture.shader_version}</div>
</div>
