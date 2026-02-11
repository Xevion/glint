<script lang="ts">
import { page } from '$app/state';
import { cfImageUrl } from '$lib/utils/image';

interface Props {
	/** Page title. Rendered as "{title} - Glint" unless `bare` is true. */
	title: string;
	/** Skip the " - Glint" suffix (for the homepage). */
	bare?: boolean;
	/** Meta description for SEO and og:description. */
	description?: string;
	/** Raw image URL (from R2/CDN). Will be transformed to 1200x630 for OG. */
	image?: string | null;
	/** Override og:type (defaults to "website"). */
	type?: string;
}

let { title, bare = false, description, image, type: ogType = 'website' }: Props = $props();

const fullTitle = $derived(bare ? title : `${title} - Glint`);
const currentUrl = $derived(page.url.href);

const ogImageUrl = $derived(
	cfImageUrl(image, { width: 1200, height: 630, fit: 'cover', quality: 85, format: 'jpeg' })
);
</script>

<svelte:head>
	<title>{fullTitle}</title>

	<!-- Primary SEO -->
	{#if description}
		<meta name="description" content={description} />
	{/if}

	<!-- Open Graph -->
	<meta property="og:title" content={fullTitle} />
	<meta property="og:type" content={ogType} />
	<meta property="og:url" content={currentUrl} />
	<meta property="og:site_name" content="Glint" />
	{#if description}
		<meta property="og:description" content={description} />
	{/if}
	{#if ogImageUrl}
		<meta property="og:image" content={ogImageUrl} />
		<meta property="og:image:width" content="1200" />
		<meta property="og:image:height" content="630" />
	{/if}

	<!-- Twitter Card -->
	<meta name="twitter:card" content={ogImageUrl ? 'summary_large_image' : 'summary'} />
	<meta name="twitter:title" content={fullTitle} />
	{#if description}
		<meta name="twitter:description" content={description} />
	{/if}
	{#if ogImageUrl}
		<meta name="twitter:image" content={ogImageUrl} />
	{/if}
</svelte:head>
