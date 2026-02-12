<script lang="ts">
import { Github, MessageCircle } from '@lucide/svelte';
</script>

<footer class="border-t border-border py-6">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
		<p class="max-w-lg text-xs text-foreground">
			Glint does not distribute shader packs. Screenshots are captured for comparison and review
			purposes. All shader packs are the property of their respective authors.
		</p>
		<div class="flex items-center gap-4 text-xs text-foreground">
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a
				href="https://github.com/Xevion/glint"
				target="_blank"
				rel="noopener noreferrer"
				class="flex items-center gap-1.5 transition-colors hover:text-foreground"
				aria-label="GitHub"
			>
				<Github class="h-4 w-4" />
				<span class="hidden sm:inline">Source</span>
			</a>
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
			<a
				href="https://github.com/Xevion/glint/issues"
				target="_blank"
				rel="noopener noreferrer"
				class="flex items-center gap-1.5 transition-colors hover:text-foreground"
				aria-label="Report an issue"
			>
				<MessageCircle class="h-4 w-4" />
				<span class="hidden sm:inline">Contact</span>
			</a>
			<span>&copy; {new Date().getFullYear()} Glint</span>
		</div>
	</div>
</footer>
