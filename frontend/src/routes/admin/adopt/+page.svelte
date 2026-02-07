<script lang="ts">
import { api } from '$lib/api';
import type { ShaderSearchResult, ShaderSearchSort } from '$lib/bindings';
import AdoptShaderDialog from '$lib/components/AdoptShaderDialog.svelte';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import * as Tabs from '$lib/components/ui/tabs';
import { cn } from '$lib/utils';
import {
	Check,
	CircleAlert,
	Download,
	ExternalLink,
	Flame,
	LoaderCircle,
	Search,
	Sparkles,
	X
} from '@lucide/svelte';

type BrowseTab = 'popular' | 'recent';

let query = $state('');
let results = $state<ShaderSearchResult[]>([]);
let totalModrinth = $state(0);
let totalCurseforge = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

let activeTab = $state<BrowseTab>('popular');
let searchActive = $state(false);
let searchQuery = $state('');

let adoptDialogOpen = $state(false);
let adoptUrl = $state<string | undefined>(undefined);

// Load popular shaders on mount
$effect(() => {
	void loadBrowse('popular');
});

// Reload when tab changes (user clicks tab)
function handleTabChange(tab: string) {
	if (searchActive) return;
	activeTab = tab as BrowseTab;
	void loadBrowse(activeTab);
}

async function loadBrowse(sort: ShaderSearchSort) {
	loading = true;
	error = null;

	const result = await api.adopt.search(undefined, 20, 0, sort);
	if (result.isOk) {
		results = result.value.results;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
		results = [];
	}
	loading = false;
}

async function handleSearch() {
	const q = query.trim();
	if (!q) return;

	loading = true;
	error = null;
	searchActive = true;
	searchQuery = q;

	const result = await api.adopt.search(q, 20, 0);
	if (result.isOk) {
		results = result.value.results;
		totalModrinth = result.value.total_modrinth;
		totalCurseforge = result.value.total_curseforge;
	} else {
		error = result.error.message;
		results = [];
	}
	loading = false;
}

function clearSearch() {
	searchActive = false;
	searchQuery = '';
	query = '';
	void loadBrowse(activeTab);
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter') {
		void handleSearch();
	}
}

function adoptFromResult(result: ShaderSearchResult) {
	adoptUrl = result.platform_url;
	adoptDialogOpen = true;
}

function handleShaderAdopted() {
	adoptUrl = undefined;
	if (searchActive) {
		void handleSearch();
	} else {
		void loadBrowse(activeTab);
	}
}

function formatNumber(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toLocaleString();
}
</script>

<div class="space-y-4">
	<header>
		<h1 class="text-2xl font-semibold">Adopt Shaders</h1>
		<p class="mt-1 text-sm text-muted-foreground">
			Browse or search for shaders on Modrinth and CurseForge to adopt into Glint.
		</p>
	</header>

	<!-- Search input -->
	<div class="flex gap-2">
		<Input
			placeholder="Search for shaders..."
			bind:value={query}
			onkeydown={handleKeydown}
			disabled={loading}
			class="max-w-md"
		/>
		<Button onclick={handleSearch} disabled={loading || !query.trim()}>
			{#if loading && searchActive}
				<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
			{:else}
				<Search class="mr-2 h-4 w-4" />
			{/if}
			Search
		</Button>
	</div>

	<!-- Tabs / Search header -->
	{#if searchActive}
		<div
			class="flex items-center gap-2 rounded-md border bg-muted/50 px-3 py-2 text-sm text-muted-foreground"
		>
			<Search class="h-4 w-4 shrink-0" />
			<span>
				Search results for "<span class="font-medium text-foreground">{searchQuery}</span>"
			</span>
			<button onclick={clearSearch} class="ml-auto rounded-sm p-0.5 hover:bg-muted">
				<X class="h-4 w-4" />
			</button>
		</div>
	{:else}
		<Tabs.Root value={activeTab} onValueChange={handleTabChange}>
			<Tabs.List>
				<Tabs.Trigger value="popular">
					<Flame class="mr-1.5 h-4 w-4" />
					Popular
				</Tabs.Trigger>
				<Tabs.Trigger value="recent">
					<Sparkles class="mr-1.5 h-4 w-4" />
					Recent
				</Tabs.Trigger>
			</Tabs.List>
		</Tabs.Root>
	{/if}

	{#if error}
		<div
			class="flex items-start gap-2 rounded-lg border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
		>
			<CircleAlert class="mt-0.5 h-4 w-4 shrink-0" />
			<div>{error}</div>
		</div>
	{/if}

	{#if loading}
		<!-- Loading skeleton -->
		<div class="space-y-2">
			{#each Array(6) as _, i (i)}
				<div class="flex items-center gap-3 rounded-lg border p-3">
					<div class="h-8 w-8 animate-pulse rounded bg-muted"></div>
					<div class="flex-1 space-y-1.5">
						<div class="h-4 w-1/3 animate-pulse rounded bg-muted"></div>
						<div class="h-3 w-2/3 animate-pulse rounded bg-muted"></div>
					</div>
					<div class="h-8 w-16 animate-pulse rounded bg-muted"></div>
				</div>
			{/each}
		</div>
	{:else if results.length === 0 && !error}
		<p class="py-8 text-center text-muted-foreground">
			{#if searchActive}
				No shaders found for "{searchQuery}".
			{:else}
				No shaders available.
			{/if}
		</p>
	{:else if results.length > 0}
		<div class="text-xs text-muted-foreground">
			{totalModrinth} Modrinth results{totalCurseforge !== null
				? `, ${totalCurseforge} CurseForge results`
				: ''}
		</div>

		<!-- Desktop table -->
		<div class="hidden rounded-md border md:block">
			<table class="w-full text-sm">
				<thead>
					<tr class="border-b bg-muted/50">
						<th class="w-10 p-3 text-left font-medium"></th>
						<th class="p-3 text-left font-medium">Name</th>
						<th class="p-3 text-left font-medium">Platform</th>
						<th class="p-3 text-left font-medium">Author</th>
						<th class="p-3 text-right font-medium">Downloads</th>
						<th class="w-24 p-3 text-right font-medium"></th>
					</tr>
				</thead>
				<tbody>
					{#each results as result (result.platform + ':' + result.platform_id)}
						<tr
							class={cn(
								'border-b transition-colors last:border-b-0 hover:bg-muted/30',
								result.adopted && 'bg-emerald-50 dark:bg-emerald-950/20'
							)}
						>
							<td class="p-3">
								{#if result.icon_url}
									<img
										src={result.icon_url}
										alt=""
										class="h-8 w-8 rounded object-cover"
									/>
								{:else}
									<div
										class="flex h-8 w-8 items-center justify-center rounded bg-muted"
									>
										<Download class="h-4 w-4 text-muted-foreground" />
									</div>
								{/if}
							</td>
							<td class="p-3">
								<div class="flex items-center gap-2">
									<span class="font-medium">{result.name}</span>
									{#if result.adopted}
										<Badge variant="secondary" class="gap-1 text-emerald-700 dark:text-emerald-400">
											<Check class="h-3 w-3" />
											Adopted
										</Badge>
									{/if}
								</div>
								<div class="line-clamp-1 text-xs text-muted-foreground">
									{result.description}
								</div>
							</td>
							<td class="p-3">
								<span
									class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium
										{result.platform === 'modrinth'
										? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
										: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
								>
									{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
								</span>
							</td>
							<td class="p-3 text-muted-foreground">{result.author}</td>
							<td class="p-3 text-right tabular-nums">
								{formatNumber(result.downloads)}
							</td>
							<td class="p-3 text-right">
								{#if result.adopted}
									<Button
										size="sm"
										variant="outline"
										href="/shaders/{result.adopted.slug}"
										class="gap-1"
									>
										<ExternalLink class="h-3.5 w-3.5" />
										View
									</Button>
								{:else}
									<Button
										size="sm"
										variant="outline"
										onclick={() => adoptFromResult(result)}
									>
										Adopt
									</Button>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<!-- Mobile cards -->
		<div class="space-y-2 md:hidden">
			{#each results as result (result.platform + ':' + result.platform_id)}
				<div
					class={cn(
						'space-y-2 rounded-lg border p-3',
						result.adopted && 'border-emerald-200 dark:border-emerald-800/40'
					)}
				>
					<div class="flex items-start gap-3">
						{#if result.icon_url}
							<img
								src={result.icon_url}
								alt=""
								class="h-10 w-10 shrink-0 rounded object-cover"
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<span class="font-medium">{result.name}</span>
								{#if result.adopted}
									<Badge variant="secondary" class="gap-1 text-emerald-700 dark:text-emerald-400">
										<Check class="h-3 w-3" />
										Adopted
									</Badge>
								{/if}
							</div>
							<div class="line-clamp-1 text-xs text-muted-foreground">
								{result.description}
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between text-xs">
						<div class="flex items-center gap-2">
							<span
								class="inline-flex items-center rounded-full px-2 py-0.5 font-medium
									{result.platform === 'modrinth'
									? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
									: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'}"
							>
								{result.platform === 'modrinth' ? 'Modrinth' : 'CurseForge'}
							</span>
							<span class="text-muted-foreground">{result.author}</span>
							<span class="tabular-nums text-muted-foreground">
								{formatNumber(result.downloads)}
							</span>
						</div>
						{#if result.adopted}
							<Button
								size="sm"
								variant="outline"
								href="/shaders/{result.adopted.slug}"
								class="gap-1"
							>
								<ExternalLink class="h-3.5 w-3.5" />
								View
							</Button>
						{:else}
							<Button
								size="sm"
								variant="outline"
								onclick={() => adoptFromResult(result)}
							>
								Adopt
							</Button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<AdoptShaderDialog
	bind:open={adoptDialogOpen}
	initialUrl={adoptUrl}
	onShaderAdopted={handleShaderAdopted}
/>
