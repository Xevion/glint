import type { CaptureItem } from '$lib/components/Lightbox.svelte';

let captures = $state<CaptureItem[]>([]);
let currentIndex = $state(0);
let isOpen = $state(false);
let closedByPopstate = false;

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
		history.pushState({ ...history.state, lightbox: true }, '');
	},
	close() {
		if (!isOpen) return;
		isOpen = false;
		if (!closedByPopstate) {
			history.back();
		}
		closedByPopstate = false;
	},
	/** Called by the popstate listener — skips history.back() since the pop already happened. */
	closeFromPopstate() {
		closedByPopstate = true;
		this.close();
	},
	navigate(index: number) {
		if (index >= 0 && index < captures.length) {
			currentIndex = index;
		}
	}
};
