import satori from 'satori';
import { Resvg } from '@resvg/resvg-js';
import { render } from 'svelte/server';
import { html } from '@xevion/satori-html';
import OgOverlay from '$lib/components/OgOverlay.svelte';
import { loadOGFonts } from './fonts';

const OG_WIDTH = 1200;
const OG_HEIGHT = 630;

export interface TextOverlayProps {
	title: string;
	subtitle?: string;
	meta?: string;
}

/**
 * Render the text overlay as a transparent PNG buffer.
 *
 * Pipeline: Svelte component → HTML string → satori VNode → SVG → resvg PNG
 */
export async function renderTextOverlay(props: TextOverlayProps): Promise<Buffer> {
	const fonts = await loadOGFonts();

	// Render Svelte component to HTML string (server-side)
	const { html: renderedHtml } = render(OgOverlay, { props });

	// Convert HTML to satori-compatible VNode
	const vnode = html(renderedHtml);

	// Generate SVG with satori (transparent background)
	const svg = await satori(vnode, {
		width: OG_WIDTH,
		height: OG_HEIGHT,
		fonts
	});

	// Rasterize SVG to PNG with transparency
	const resvg = new Resvg(svg, {
		fitTo: { mode: 'width', value: OG_WIDTH }
	});
	const pngData = resvg.render();
	return Buffer.from(pngData.asPng());
}
