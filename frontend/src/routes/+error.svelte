<script lang="ts">
import { dev } from '$app/environment';
import { resolve } from '$app/paths';
import { page } from '$app/state';
import ErrorCard from '$lib/components/ErrorCard.svelte';
import { Button } from '$lib/components/ui/button';
import { fade } from 'svelte/transition';

const errorTitles: Record<number, { title: string; description: string }> = {
	400: {
		title: 'Bad Request',
		description: 'The request was malformed or invalid.'
	},
	403: {
		title: 'Forbidden',
		description: "You don't have permission to access this resource."
	},
	404: {
		title: 'Page Not Found',
		description: "The page you're looking for doesn't exist or has been moved."
	},
	410: {
		title: 'Gone',
		description: 'This content has been permanently removed.'
	},
	502: {
		title: 'Backend Unavailable',
		description: 'The backend service is unreachable. Please try again later.'
	},
	503: {
		title: 'Service Unavailable',
		description: 'The service is temporarily unavailable. Please try again later.'
	}
};

let status = $derived(page.status);
let error = $derived(page.error);
let source = $derived(error?.source ?? 'server');

let info = $derived.by(() => {
	if (status === 404) return errorTitles[404];
	if (source === 'client')
		return { title: 'Client Error', description: 'An error occurred in the browser.' };
	return (
		errorTitles[status] ?? {
			title: 'Server Error',
			description: 'Something went wrong while processing your request.'
		}
	);
});

let statusLabel = $derived.by(() => {
	if (source === 'client' && status === 500) return 'CLIENT';
	return String(status);
});
</script>

<div class="py-16" in:fade={{ duration: 200 }}>
	<div class="mx-auto max-w-2xl px-4">
		<ErrorCard
			title={info.title}
			description={info.description}
			{statusLabel}
			message={error?.message}
			errorId={error?.errorId}
			requestId={error?.requestId}
			timestamp={error?.timestamp}
			stack={error?.stack}
			code={error?.code}
			detail={error?.detail}
			defaultStackOpen={dev}
		>
			{#snippet actions()}
				<Button href={resolve('/', {})} variant="default" size="sm">Go Home</Button>
			{/snippet}
		</ErrorCard>
	</div>
</div>
