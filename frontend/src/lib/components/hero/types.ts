export type Orientation = 'vertical' | 'horizontal' | 'diagonal';

/** Degrees of skew for diagonal clip-path and divider line. Shared across hero components. */
export const SKEW_DEG = 8;

/** Data for one side of a comparison slider. Matches FeaturedSideNode GraphQL shape. */
export interface SliderSide {
	imagePath: string;
	thumbhash?: string | null;
	shaderName: string;
	shaderSlug: string;
	shaderAuthor?: string | null;
	shaderVersion: string;
	sceneName: string;
}

/** Empty SliderSide for initialization before data loads. */
export const EMPTY_SIDE: SliderSide = {
	imagePath: '',
	shaderName: '',
	shaderSlug: '',
	shaderVersion: '',
	sceneName: ''
};

/** A featured shader comparison pair. Matches FeaturedPairNode GraphQL shape. */
export interface HeroPair {
	left: SliderSide;
	right: SliderSide;
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
