import type { Shader } from '$lib/bindings';

function createComparisonStore() {
	let leftShader = $state<Shader | null>(null);
	let rightShader = $state<Shader | null>(null);
	let _selectionMode = $state(false);

	return {
		get left(): Shader | null {
			return leftShader;
		},
		get right(): Shader | null {
			return rightShader;
		},
		get canCompare(): boolean {
			return leftShader !== null && rightShader !== null;
		},
		get selectionMode(): boolean {
			return _selectionMode;
		},
		set selectionMode(value: boolean) {
			_selectionMode = value;
			if (!value) {
				leftShader = null;
				rightShader = null;
			}
		},

		isSelected(id: string): boolean {
			return leftShader?.id === id || rightShader?.id === id;
		},

		hasSelection(): boolean {
			return leftShader !== null || rightShader !== null;
		},

		/** Assign shader to the first empty slot, or replace right if both full. */
		select(shader: Shader) {
			if (leftShader?.id === shader.id || rightShader?.id === shader.id) {
				// Already selected — deselect
				if (leftShader?.id === shader.id) leftShader = null;
				else rightShader = null;
				return;
			}
			if (leftShader === null) leftShader = shader;
			else if (rightShader === null) rightShader = shader;
			else rightShader = shader;
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
		}
	};
}

export const comparisonStore = createComparisonStore();
