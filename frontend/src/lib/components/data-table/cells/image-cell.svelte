<script lang="ts">
interface Props {
	url: string;
	sizeClass: string;
	roundedClass: string;
}

let { url, sizeClass, roundedClass }: Props = $props();
</script>

<img src={url} alt="" loading="lazy" class="{sizeClass} {roundedClass} shrink-0 object-cover pointer-events-none" />
