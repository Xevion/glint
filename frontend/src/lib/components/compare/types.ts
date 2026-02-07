import type { Shader } from '$lib/bindings';

/** Display-only shader info for comparison overlays */
export type ShaderDisplayInfo = Pick<Shader, 'name'> & {
	version: string;
	author: string | null;
	profile: string | null;
};

/** Comparison display modes */
export type CompareMode = 'slider' | 'split' | 'toggle';

/** Per-element luminance samples for adaptive text coloring */
export interface ElementLuminances {
	name?: number;
	meta?: number;
	author?: number;
}

/** Bounding rectangles for overlay text elements */
export interface ElementBounds {
	name: DOMRect | null;
	meta: DOMRect | null;
	author: DOMRect | null;
}

/** Image option for ImagePicker */
export interface ImageOption {
	url: string;
	label: string;
	thumbhash?: string | null;
}
