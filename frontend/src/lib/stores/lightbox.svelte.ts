import type { CaptureItem } from '$lib/components/Lightbox.svelte';

let captures = $state<CaptureItem[]>([]);
let currentIndex = $state(0);
let isOpen = $state(false);

export const lightbox = {
	get isOpen() {
		return isOpen;
	},
	get captures() {
		return captures;
	},
	get currentIndex() {
		return currentIndex;
	},

	open(items: CaptureItem[], startIndex = 0) {
		captures = items;
		currentIndex = startIndex;
		isOpen = true;
	},
	close() {
		isOpen = false;
	},
	navigate(index: number) {
		if (index >= 0 && index < captures.length) {
			currentIndex = index;
		}
	}
};
