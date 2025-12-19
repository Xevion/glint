/**
 * Store for managing comparison selection state across shader and scene cards.
 * Tracks which items are selected for comparison and provides reactive state
 * for UI behavior changes (checkbox visibility, click behavior, etc.)
 */

import { SvelteSet } from 'svelte/reactivity';

function createComparisonStore() {
	const selectedShaders = new SvelteSet<string>();
	const selectedScenes = new SvelteSet<string>();

	return {
		get selectedShaders() {
			return selectedShaders;
		},
		get selectedScenes() {
			return selectedScenes;
		},
		get hasShaderSelection() {
			return selectedShaders.size > 0;
		},
		get hasSceneSelection() {
			return selectedScenes.size > 0;
		},
		get shaderCount() {
			return selectedShaders.size;
		},
		get sceneCount() {
			return selectedScenes.size;
		},

		isShaderSelected(id: string): boolean {
			return selectedShaders.has(id);
		},

		isSceneSelected(id: string): boolean {
			return selectedScenes.has(id);
		},

		toggleShader(id: string) {
			if (selectedShaders.has(id)) {
				selectedShaders.delete(id);
			} else {
				selectedShaders.add(id);
			}
		},

		toggleScene(id: string) {
			if (selectedScenes.has(id)) {
				selectedScenes.delete(id);
			} else {
				selectedScenes.add(id);
			}
		},

		clearShaders() {
			selectedShaders.clear();
		},

		clearScenes() {
			selectedScenes.clear();
		},

		clearAll() {
			selectedShaders.clear();
			selectedScenes.clear();
		}
	};
}

export const comparisonStore = createComparisonStore();
