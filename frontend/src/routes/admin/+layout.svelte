<script lang="ts">
import type { Snippet } from 'svelte';
import { page } from '$app/state';
import { Button } from '$lib/components/ui/button';
import { ShieldX } from '@lucide/svelte';
import { fly } from 'svelte/transition';
import type { LayoutData } from './$types';

interface Props {
	data: LayoutData;
	children: Snippet;
}

let { data, children }: Props = $props();

const isAdmin = $derived(data.isAdmin);
const loginUrl = $derived(`/login?redirect=${encodeURIComponent(page.url.pathname)}`);
</script>

{#if isAdmin}
	{@render children()}
{:else}
	<div class="flex min-h-[60vh] items-center justify-center">
		<div
			in:fly={{ y: 20, duration: 400 }}
			class="w-full max-w-sm rounded-xl border border-border bg-card p-8"
		>
			<div class="text-center">
				<ShieldX class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
				<h1 class="mb-2 text-2xl font-bold text-card-foreground">Unauthorized</h1>
				<p class="mb-6 text-muted-foreground">
					You need to be signed in as an admin to access this page.
				</p>
				<Button href={loginUrl} class="w-full">Sign in</Button>
			</div>
		</div>
	</div>
{/if}
