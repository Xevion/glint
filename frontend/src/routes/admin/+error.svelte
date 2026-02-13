<script lang="ts">
import { dev } from '$app/environment';
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
		title: 'Not Found',
		description: "The page you're looking for doesn't exist."
	},
	500: {
		title: 'Server Error',
		description: 'Something went wrong while processing your request.'
	}
};

let status = $derived(page.status);
let error = $derived(page.error);

let info = $derived(
	errorTitles[status] ?? {
		title: 'Error',
		description: 'Something went wrong.'
	}
);
</script>

<div class="py-12" in:fade={{ duration: 200 }}>
	<div class="mx-auto max-w-2xl">
		<ErrorCard
			title={info.title}
			description={info.description}
			statusLabel={String(status)}
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
				<Button href="/admin" variant="default" size="sm">Back to Dashboard</Button>
			{/snippet}
		</ErrorCard>
	</div>
</div>
