<script lang="ts">
import { fly, fade } from 'svelte/transition';
import { resolve } from '$app/paths';
import { Button } from '$lib/components/ui/button';

const message = 'Comparison features coming soon in alpha development';
</script>

<div class="container mx-auto px-4 py-16">
	<div class="mx-auto max-w-2xl text-center">
		<div in:fly={{ y: -10, duration: 400 }} class="mb-8">
			<svg
				class="mx-auto mb-4 h-24 w-24 text-muted-foreground/30"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
				/>
			</svg>
			<h1 class="mb-3 text-3xl font-bold text-foreground">Compare Shaders & Scenes</h1>
			<p class="text-lg text-muted-foreground">{message}</p>
		</div>

		<div
			in:fly={{ y: 10, duration: 400, delay: 100 }}
			class="mb-8 rounded-xl border border-border bg-card p-6 text-left"
		>
			<h2 class="mb-4 text-xl font-semibold text-card-foreground">Planned Features</h2>
			<ul class="space-y-2 text-sm text-muted-foreground">
				<li class="flex items-start gap-2">
					<svg
						class="mt-0.5 h-5 w-5 flex-shrink-0 text-primary"
						fill="currentColor"
						viewBox="0 0 20 20"
					>
						<path
							fill-rule="evenodd"
							d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
							clip-rule="evenodd"
						/>
					</svg>
					<span>Side-by-side shader comparison in the same scene</span>
				</li>
				<li class="flex items-start gap-2">
					<svg
						class="mt-0.5 h-5 w-5 flex-shrink-0 text-primary"
						fill="currentColor"
						viewBox="0 0 20 20"
					>
						<path
							fill-rule="evenodd"
							d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
							clip-rule="evenodd"
						/>
					</svg>
					<span>Performance metrics comparison (FPS, frame time, GPU usage)</span>
				</li>
				<li class="flex items-start gap-2">
					<svg
						class="mt-0.5 h-5 w-5 flex-shrink-0 text-primary"
						fill="currentColor"
						viewBox="0 0 20 20"
					>
						<path
							fill-rule="evenodd"
							d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
							clip-rule="evenodd"
						/>
					</svg>
					<span>Interactive slider to compare screenshots</span>
				</li>
				<li class="flex items-start gap-2">
					<svg
						class="mt-0.5 h-5 w-5 flex-shrink-0 text-primary"
						fill="currentColor"
						viewBox="0 0 20 20"
					>
						<path
							fill-rule="evenodd"
							d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
							clip-rule="evenodd"
						/>
					</svg>
					<span>Save and share comparison links</span>
				</li>
			</ul>
		</div>

		<div
			in:fade={{ duration: 300, delay: 200 }}
			class="flex flex-col gap-3 sm:flex-row sm:justify-center"
		>
			<Button href={resolve('/shaders', {})} variant="default">Browse Shaders</Button>
			<Button href={resolve('/scenes', {})} variant="outline">View Scenes</Button>
		</div>
	</div>
</div>
