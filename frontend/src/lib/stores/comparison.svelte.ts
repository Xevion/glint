/** Minimal shader identity needed by the comparison store and its consumers. */
export interface ComparisonShader {
	id: string;
	name: string;
}

function createComparisonStore() {
	let leftShader = $state<ComparisonShader | null>(null);
	let rightShader = $state<ComparisonShader | null>(null);
	let _selectionMode = $state(false);

	return {
		get left(): ComparisonShader | null {
			return leftShader;
		},
		get right(): ComparisonShader | null {
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
		select(shader: ComparisonShader) {
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

		setLeft(shader: ComparisonShader | null) {
			leftShader = shader;
		},

		setRight(shader: ComparisonShader | null) {
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
