/**
 * Store for managing comparison selection state across shader and scene cards.
 * Tracks which items are selected for comparison and provides reactive state
 * for UI behavior changes (checkbox visibility, click behavior, etc.)
 */

function createComparisonStore() {
	let selectedShaders = $state<Set<string>>(new Set());
	let selectedScenes = $state<Set<string>>(new Set());

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
				selectedShaders = new Set(selectedShaders);
			} else {
				selectedShaders = new Set(selectedShaders).add(id);
			}
		},

		toggleScene(id: string) {
			if (selectedScenes.has(id)) {
				selectedScenes.delete(id);
				selectedScenes = new Set(selectedScenes);
			} else {
				selectedScenes = new Set(selectedScenes).add(id);
			}
		},

		clearShaders() {
			selectedShaders = new Set();
		},

		clearScenes() {
			selectedScenes = new Set();
		},

		clearAll() {
			selectedShaders = new Set();
			selectedScenes = new Set();
		}
	};
}

export const comparisonStore = createComparisonStore();
