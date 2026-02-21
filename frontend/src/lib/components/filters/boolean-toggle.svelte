<script lang="ts">
import * as Checkbox from '$lib/components/ui/checkbox';

interface Props {
	label: string;
	checked: boolean | null;
	class?: string;
}
let { label, checked = $bindable(false), class: className }: Props = $props();
</script>

<label class="flex items-center gap-2 text-sm {className ?? ''}">
	<Checkbox.Root checked={checked ?? false} onCheckedChange={(v) => (checked = v === true)} />
	<span>{label}</span>
</label>
