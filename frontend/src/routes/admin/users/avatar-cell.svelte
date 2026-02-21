<script lang="ts">
import ImageCell from '$lib/components/data-table/cells/image-cell.svelte';
import ImageFallbackCell from '$lib/components/data-table/cells/image-fallback-cell.svelte';
import type { User } from '$lib/bindings';
import { CircleUser } from '@lucide/svelte';

interface Props {
	user: User;
}

let { user }: Props = $props();

const avatarUrl = $derived(
	user.discord_avatar
		? `https://cdn.discordapp.com/avatars/${user.discord_id}/${user.discord_avatar}.png?size=40`
		: null
);
</script>

{#if avatarUrl}
	<ImageCell url={avatarUrl} sizeClass="h-8 w-8" roundedClass="rounded-full" />
{:else}
	<ImageFallbackCell icon={CircleUser} sizeClass="h-8 w-8" roundedClass="rounded-full" />
{/if}
