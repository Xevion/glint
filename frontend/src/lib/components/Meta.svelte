<script lang="ts">
import { page } from '$app/state';
import { imageUrl } from '$lib/utils/image';

interface Props {
	/** Page title. Rendered as "{title} - Glint" unless `bare` is true. */
	title: string;
	/** Skip the " - Glint" suffix (for the homepage). */
	bare?: boolean;
	/** Meta description for SEO and og:description. */
	description?: string;
	/** Image path (S3 key). Will be transformed to 1200x630 for OG. */
	image?: string | null;
	/**
	 * Path to a composite OG image endpoint (e.g. "/og/shader/bsl/og.png").
	 * When set, this takes precedence over the `image` prop.
	 * Resolved to an absolute URL using the current page origin.
	 */
	ogImagePath?: string | null;
	/** Override og:type (defaults to "website"). */
	type?: string;
}

let {
	title,
	bare = false,
	description,
	image,
	ogImagePath,
	type: ogType = 'website'
}: Props = $props();

const fullTitle = $derived(bare ? title : `${title} - Glint`);
const currentUrl = $derived(page.url.href);

const ogImageUrl = $derived.by(() => {
	// Composite OG image path takes precedence
	if (ogImagePath) return `${page.url.origin}${ogImagePath}`;
	// Fall back to transformed image from path
	if (image) {
		return imageUrl(image, { width: 1200, height: 630, fit: 'cover', quality: 85, format: 'jpeg' });
	}
	return null;
});

// Composite OG images are JPEG; Cloudflare-transformed images use JPEG format param
const ogImageType = $derived(ogImageUrl ? 'image/jpeg' : null);
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
		{#if ogImageType}
			<meta property="og:image:type" content={ogImageType} />
		{/if}
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
