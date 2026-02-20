import type { ApiError } from '$lib/api/errors';
import type { Result } from 'true-myth';

interface AdminActionConfig {
	/** The async operation to perform */
	action: () => Promise<Result<unknown, ApiError>>;
	/** Called on success (e.g. `() => void goto('/admin/shaders')`) */
	onSuccess: () => void;
	/** Write errors to a shared error state (e.g. `(msg) => form.error = msg`) */
	setError: (message: string) => void;
}

/**
 * Composable for one-off admin actions (delete, disable, reactivate, etc.).
 *
 * Manages loading state and delegates error display to a shared setter,
 * typically the form's error state.
 *
 * @example
 * ```ts
 * const deleteAction = createAction({
 *   action: () => api.admin.deleteShader(shader.id),
 *   onSuccess: () => void goto('/admin/shaders'),
 *   setError: (msg) => (form.error = msg),
 * });
 * // Template: disabled={deleteAction.loading}
 * // Template: onclick={deleteAction.execute}
 * ```
 */
export function createAction(config: AdminActionConfig) {
	let loading = $state(false);

	async function execute() {
		loading = true;
		try {
			const result = await config.action();
			result.match({
				Ok: () => config.onSuccess(),
				Err: (err) => config.setError(err.message)
			});
		} finally {
			loading = false;
		}
	}

	return {
		/** Whether the action is in progress */
		get loading() {
			return loading;
		},
		/** Execute the action */
		execute
	};
}
