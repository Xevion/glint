<script lang="ts">
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';
	import { fly, fade } from 'svelte/transition';
	import { Button } from '$lib/components/ui/button';

	const statusMessages: Record<number, { title: string; description: string }> = {
		404: {
			title: 'Page Not Found',
			description: "The page you're looking for doesn't exist or has been moved."
		},
		403: {
			title: 'Forbidden',
			description: "You don't have permission to access this resource."
		},
		500: {
			title: 'Internal Server Error',
			description: 'Something went wrong on our end. Please try again later.'
		},
		503: {
			title: 'Service Unavailable',
			description: 'The service is temporarily unavailable. Please try again later.'
		}
	};

	$: status = $page.status;
	$: message = statusMessages[status] ?? {
		title: 'Error',
		description: $page.error?.message ?? 'An unexpected error occurred.'
	};
</script>

<div class="container mx-auto px-4 py-16">
	<div class="mx-auto max-w-2xl text-center">
		<div in:fly={{ y: -20, duration: 400 }} class="mb-8">
			<div class="text-[10rem] leading-none font-bold text-primary">{status}</div>
		</div>

		<h1 in:fly={{ y: 10, duration: 400, delay: 100 }} class="mb-6 text-3xl font-bold">
			{message.title}
		</h1>

		<p
			in:fade={{ duration: 300, delay: 200 }}
			class="mb-12 text-xl text-foreground/70 dark:text-muted-foreground"
		>
			{message.description}
		</p>

		<div in:fade={{ duration: 300, delay: 300 }}>
			<Button href={resolve('/')} variant="default" size="lg">Go Home</Button>
		</div>
	</div>
</div>
