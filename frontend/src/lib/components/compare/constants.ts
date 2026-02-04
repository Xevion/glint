/** Luminance thresholds for text color switching (ITU-R BT.709) */
export const LUMINANCE = {
	/** Switch to dark text when background luminance exceeds this */
	HIGH_THRESHOLD: 0.55,
	/** Switch to light text when background luminance falls below this */
	LOW_THRESHOLD: 0.45,
	/** Bias toward white text (subtracted from luminance before comparison) */
	BIAS: 0.1,
	/** Animation chase factor (higher = faster transitions) */
	CHASE_FACTOR: 8,
	/** Minimum animation speed (prevents stalling near target) */
	MIN_SPEED: 2,
	/** Epsilon for animation completion detection */
	EPSILON: 0.005,
	/** Throttle interval for luminance sampling (ms) */
	SAMPLE_THROTTLE_MS: 50
} as const;

/** Slider mode visibility thresholds (percentage) */
export const SLIDER = {
	/** Hide left overlay when slider position < this value */
	HIDE_LEFT_THRESHOLD: 30,
	/** Hide right overlay when slider position > this value */
	HIDE_RIGHT_THRESHOLD: 70
} as const;

/** Zoom constraints for SplitCanvas */
export const ZOOM = {
	/** Maximum zoom level */
	MAX: 4,
	/** Double-tap zoom multiplier (relative to cover zoom) */
	DOUBLE_TAP_MULTIPLIER: 2,
	/** Threshold for considering "zoomed in" (relative to cover zoom) */
	ZOOMED_IN_THRESHOLD: 1.1
} as const;

/** Touch interaction timing (ms) */
export const TOUCH = {
	/** Window for detecting double-tap */
	DOUBLE_TAP_WINDOW: 300
} as const;

/** Sampling region size for luminance detection (pixels from corner) */
export const SAMPLING = {
	/** Width of the corner region to sample */
	WIDTH: 150,
	/** Height of the corner region to sample */
	HEIGHT: 60
} as const;
