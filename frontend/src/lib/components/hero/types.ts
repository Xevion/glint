import type { FeaturedPair } from '$lib/bindings';

export type Orientation = 'vertical' | 'horizontal' | 'diagonal';

/** Degrees of skew for diagonal clip-path and divider line. Shared across hero components. */
export const SKEW_DEG = 8;

/** Data for one side of a comparison slider. */
export interface SliderSide {
	image: string;
	thumbhash?: string | null;
	label: string;
	slug: string;
	author?: string | null;
	version: string;
}

/** Empty SliderSide for initialization before data loads. */
export const EMPTY_SIDE: SliderSide = {
	image: '',
	label: '',
	slug: '',
	version: ''
};

/** Extract left and right SliderSide from a FeaturedPair. */
export function pairToSides(pair: FeaturedPair): { left: SliderSide; right: SliderSide } {
	return {
		left: {
			image: pair.left_image_url,
			thumbhash: pair.left_thumbhash ?? null,
			label: pair.left_shader_name,
			slug: pair.left_shader_slug,
			author: pair.left_shader_author,
			version: pair.left_shader_version
		},
		right: {
			image: pair.right_image_url,
			thumbhash: pair.right_thumbhash ?? null,
			label: pair.right_shader_name,
			slug: pair.right_shader_slug,
			author: pair.right_shader_author,
			version: pair.right_shader_version
		}
	};
}

/**
 * Compute CSS clip-paths for left and right image layers.
 * Pure function — extracted for testability.
 */
export function computeClipPaths(
	position: number,
	orientation: Orientation,
	skewDeg: number = SKEW_DEG
): { left: string; right: string } {
	const pos = position * 100;

	if (orientation === 'vertical') {
		return {
			left: `inset(0 ${100 - pos}% 0 0)`,
			right: `inset(0 0 0 ${pos}%)`
		};
	} else if (orientation === 'horizontal') {
		return {
			left: `inset(0 0 ${100 - pos}% 0)`,
			right: `inset(${pos}% 0 0 0)`
		};
	} else {
		const l = pos - skewDeg;
		const r = pos + skewDeg;
		return {
			left: `polygon(0% 0%, ${r}% 0%, ${l}% 100%, 0% 100%)`,
			right: `polygon(${r}% 0%, 100% 0%, 100% 100%, ${l}% 100%)`
		};
	}
}
