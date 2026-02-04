/**
 * Store for managing shader comparison state.
 *
 * Provides two APIs:
 * 1. New: Typed left/right Shader slots for the compare page
 * 2. Legacy: Selection-based toggle API for card components (Phase 2 will migrate these)
 */

import type { Shader } from '$lib/bindings';
import { SvelteSet } from 'svelte/reactivity';

function createComparisonStore() {
	// New API: typed comparison slots
	let leftShader = $state<Shader | null>(null);
	let rightShader = $state<Shader | null>(null);

	// Legacy API: selection sets for card multi-select behavior
	const selectedShaders = new SvelteSet<string>();
	const selectedScenes = new SvelteSet<string>();

	return {
		// === New API (typed slots) ===
		get left(): Shader | null {
			return leftShader;
		},
		get right(): Shader | null {
			return rightShader;
		},
		get canCompare(): boolean {
			return leftShader !== null && rightShader !== null;
		},

		setLeft(shader: Shader | null) {
			leftShader = shader;
		},

		setRight(shader: Shader | null) {
			rightShader = shader;
		},

		swap() {
			const temp = leftShader;
			leftShader = rightShader;
			rightShader = temp;
		},

		clear() {
			leftShader = null;
			rightShader = null;
		},

		// === Legacy API (selection sets) - will be removed in Phase 2 ===
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
			leftShader = null;
			rightShader = null;
		}
	};
}

export const comparisonStore = createComparisonStore();
